#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use winit::{
    event::{Event, KeyEvent, WindowEvent, MouseButton, ElementState},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{WindowBuilder, Icon},
    dpi::PhysicalPosition,
};
#[cfg(not(target_os = "linux"))]
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, MenuEvent}, TrayIconEvent};
use std::sync::{Arc, Mutex};


include!(concat!(env!("OUT_DIR"), "/squirrel_data.rs"));

fn main() {
    let img = image::load_from_memory(SQUIRREL_PNG).unwrap();
    let (width, height) = (img.width(), img.height());
    
    let event_loop = EventLoop::new().unwrap();
    let icon_img = img.resize(32, 32, image::imageops::FilterType::Lanczos3).to_rgba8();
    let icon = Icon::from_rgba(icon_img.clone().into_raw(), 32, 32).unwrap();
    
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(width * 3 / 4, height * 3 / 4))
        .with_decorations(false)
        .with_resizable(false)
        .with_window_icon(Some(icon.clone()))
        .build(&event_loop)
        .unwrap();
    window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
    
    let visible = Arc::new(Mutex::new(true));
    let dragging = Arc::new(Mutex::new(false));
    let drag_start = Arc::new(Mutex::new((0.0, 0.0)));
    let bouncing = Arc::new(Mutex::new(false));
    let velocity = Arc::new(Mutex::new((2.0, 2.0)));
    let last_time = Arc::new(Mutex::new(std::time::Instant::now()));

    
    #[cfg(not(target_os = "linux"))]
    let quit_item = MenuItem::new("Quit", true, None);
    #[cfg(not(target_os = "linux"))]
    let menu = Menu::new();
    #[cfg(not(target_os = "linux"))]
    menu.append(&quit_item).unwrap();
    
    #[cfg(not(target_os = "linux"))]
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(tray_icon::Icon::from_rgba(icon_img.into_raw(), 32, 32).unwrap())
        .build()
        .unwrap();
    
    #[cfg(not(target_os = "linux"))]
    let menu_channel = TrayIconEvent::receiver();
    #[cfg(not(target_os = "linux"))]
    let menu_event_channel = MenuEvent::receiver();
    #[cfg(not(target_os = "linux"))]
    let quit_id = quit_item.id();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap()) }.unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor::default(),
        None,
    )).unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0];
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: width * 3 / 4,
        height: height * 3 / 4,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let img_rgba = img.to_rgba8();
    let texture_size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: None,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
        label: None,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        #[cfg(not(target_os = "linux"))]
        if let Ok(event) = menu_channel.try_recv() {
            if let TrayIconEvent::Click { .. } = event {
                let mut vis = visible.lock().unwrap();
                *vis = !*vis;
                if *vis {
                    window.set_visible(true);
                    window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                    window.focus_window();
                } else {
                    window.set_visible(false);
                }
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        if let Ok(event) = menu_event_channel.try_recv() {
            if event.id == quit_id {
                elwt.exit();
                return;
            }
        }
        
        match event {
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(NamedKey::F9), state: ElementState::Pressed, .. }, .. }, .. } => {
                let mut vis = visible.lock().unwrap();
                *vis = !*vis;
                if *vis {
                    window.set_visible(true);
                    window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                    window.focus_window();
                } else {
                    window.set_visible(false);
                }
            },
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. }, .. } => {
                match logical_key {
                    Key::Character(c) if c == "d" => {
                        let mut bounce = bouncing.lock().unwrap();
                        *bounce = !*bounce;
                        *last_time.lock().unwrap() = std::time::Instant::now();
                    },
                    Key::Named(key) => {
                        if !*bouncing.lock().unwrap() {
                            if let Ok(pos) = window.outer_position() {
                                let new_pos = match key {
                                    NamedKey::ArrowUp => PhysicalPosition::new(pos.x, pos.y - 10),
                                    NamedKey::ArrowDown => PhysicalPosition::new(pos.x, pos.y + 10),
                                    NamedKey::ArrowLeft => PhysicalPosition::new(pos.x - 10, pos.y),
                                    NamedKey::ArrowRight => PhysicalPosition::new(pos.x + 10, pos.y),
                                    _ => pos,
                                };
                                if new_pos != pos {
                                    window.set_outer_position(new_pos);
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            Event::WindowEvent { event: WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Pressed, .. }, .. } => {
                *dragging.lock().unwrap() = true;
                if let Ok(pos) = window.outer_position() {
                    *drag_start.lock().unwrap() = (pos.x as f64, pos.y as f64);
                }
            },
            Event::WindowEvent { event: WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Released, .. }, .. } => {
                *dragging.lock().unwrap() = false;
            },
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if *dragging.lock().unwrap() {
                    let start = *drag_start.lock().unwrap();
                    let new_pos = PhysicalPosition::new(
                        (start.0 + position.x - (width * 3 / 4) as f64 / 2.0) as i32,
                        (start.1 + position.y - (height * 3 / 4) as f64 / 2.0) as i32
                    );
                    window.set_outer_position(new_pos);
                }
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                if *visible.lock().unwrap() {
                    let output = surface.get_current_texture().unwrap();
                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        render_pass.set_pipeline(&render_pipeline);
                        render_pass.set_bind_group(0, &bind_group, &[]);
                        render_pass.draw(0..6, 0..1);
                    }
                    queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }
            }
            Event::AboutToWait => {
                if *visible.lock().unwrap() {
                    if *bouncing.lock().unwrap() {
                        let now = std::time::Instant::now();
                        let mut last = last_time.lock().unwrap();
                        let dt = now.duration_since(*last).as_secs_f64();
                        *last = now;
                        
                        if let Ok(pos) = window.outer_position() {
                            let mut vel = velocity.lock().unwrap();
                            let screen_width = 1920.0; // Approximate screen width
                            let screen_height = 1080.0; // Approximate screen height
                            let win_width = (width * 3 / 4) as f64;
                            let win_height = (height * 3 / 4) as f64;
                            
                            let mut new_x = pos.x as f64 + vel.0 * dt * 60.0;
                            let mut new_y = pos.y as f64 + vel.1 * dt * 60.0;
                            
                            if new_x <= 0.0 || new_x + win_width >= screen_width {
                                vel.0 = -vel.0;
                                new_x = new_x.clamp(0.0, screen_width - win_width);
                            }
                            if new_y <= 0.0 || new_y + win_height >= screen_height {
                                vel.1 = -vel.1;
                                new_y = new_y.clamp(0.0, screen_height - win_height);
                            }
                            
                            window.set_outer_position(PhysicalPosition::new(new_x as i32, new_y as i32));
                        }
                    }
                    window.request_redraw();
                }
            },
            _ => {}
        }
    }).unwrap();
}