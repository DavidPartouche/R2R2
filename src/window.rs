use std::os::raw::c_void;

use winit::error::OsError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::platform::windows::WindowExtWindows;
use winit::window as winit_window;
use winit::window::WindowBuilder;

pub struct Window {
    event_loop: EventLoop<()>,
    window: winit_window::Window,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Window, OsError> {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size((width, height).into())
            .with_resizable(false)
            .build(&event_loop)?;

        Ok(Window { event_loop, window })
    }

    pub fn hwnd(&self) -> *mut c_void {
        self.window.hwnd()
    }

    pub fn size(&self) -> Size {
        let dpi = self.window.hidpi_factor();
        let physical_size = self.window.inner_size().to_physical(dpi);
        Size {
            width: physical_size.width as u32,
            height: physical_size.height as u32,
        }
    }

    pub fn run<T>(self, mut draw: T)
    where
        T: FnMut(),
    {
        let mut event_loop = self.event_loop;
        let window = self.window;

        event_loop.run_return(move |event, _, control_flow| {
            match event {
                Event::EventsCleared => {
                    // Application update code
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Redraw the application
                    draw();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => *control_flow = ControlFlow::Poll,
            }
        });
    }
}
