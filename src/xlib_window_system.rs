#![allow(non_upper_case_globals)]

extern crate libc;
extern crate xlib;
extern crate xinerama;

use self::libc::{ c_char, c_int, c_uint, c_void };
use self::libc::funcs::c95::stdlib::malloc;
use self::xlib::{
    Display,
    Window,
    XCrossingEvent,
    XSetWindowBorder,
    XSetWindowBorderWidth,
    XDefaultRootWindow,
    XDefaultScreenOfDisplay,
    XDisplayWidth,
    XDisplayHeight,
    XEnterWindowEvent,
    XFetchName,
    XLeaveWindowEvent,
    XMapRequestEvent,
    XMapWindow,
    XMotionEvent,
    XMoveWindow,
    XNextEvent,
    XOpenDisplay,
    XPending,
    XQueryTree,
    XResizeWindow,
    XRootWindowOfScreen,
    XScreenCount,
    XSelectInput,
    XSync
};
use self::xinerama::{
    XineramaQueryScreens,
    XineramaScreenInfo
};

use std::ptr::null_mut;
use std::mem::transmute;
use std::mem::uninitialized;
use std::str::raw::c_str_to_static_slice;
use std::slice::raw::buf_as_slice;

use window_system::{ Rectangle, WindowSystem, WindowSystemEvent };
use window_system::{
    Enter,
    Leave,
    WindowCreated,
    WindowDestroyed,
    UnknownEvent
};

const KeyPress               : uint =  2;
const KeyRelease             : uint =  3;
const ButtonPress            : uint =  4;
const ButtonRelease          : uint =  5;
const MotionNotify           : uint =  6;
const EnterNotify            : uint =  7;
const LeaveNotify            : uint =  8;
const FocusIn                : uint =  9;
const FocusOut               : uint = 10;
const KeymapNotify           : uint = 11;
const Expose                 : uint = 12;
const GraphicsExpose         : uint = 13;
const NoExpose               : uint = 14;
const VisibilityNotify       : uint = 15;
const CreateNotify           : uint = 16;
const DestroyNotify          : uint = 17;
const UnmapNotify            : uint = 18;
const MapNotify              : uint = 19;
const MapRequest             : uint = 20;
const ReparentNotify         : uint = 21;
const ConfigureNotify        : uint = 22;
const ConfigureRequest       : uint = 23;
const GravityNotify          : uint = 24;
const ResizeRequest          : uint = 25;
const CirculateNotify        : uint = 26;
const CirculateRequest       : uint = 27;
const PropertyNotify         : uint = 28;
const SelectionClear         : uint = 29;
const SelectionRequest       : uint = 30;
const SelectionNotify        : uint = 31;
const ColormapNotify         : uint = 32;
const ClientMessage          : uint = 33;
const MappingNotify          : uint = 34;

pub struct XlibWindowSystem {
    display: *mut Display,
    root:    Window,
    event:   *mut c_void
}

impl XlibWindowSystem {
    pub fn new() -> XlibWindowSystem {
        unsafe {
            let display = XOpenDisplay(null_mut());
            let screen  = XDefaultScreenOfDisplay(display);
            let root    = XRootWindowOfScreen(screen);

            XSelectInput(display, root, 0x180030);
            XSync(display, 0);

            XlibWindowSystem {
                display: display,
                root:    root,
                event:   malloc(256)
            }
        }
    }

    fn get_event_as<T>(&self) -> &T {
        unsafe {
            let event_ptr : *const T = transmute(self.event);
            let ref event = *event_ptr;
            event
        }
    }
}

impl WindowSystem for XlibWindowSystem {
    fn get_screen_infos(&self) -> Vec<Rectangle> {
        unsafe {
            let mut num : c_int = 0;
            let screen_ptr = XineramaQueryScreens(self.display, &mut num);

            // If xinerama is not active, just return the default display
            // dimensions and "emulate" xinerama.
            if num == 0 {
                return vec!(Rectangle(0, 0, 
                                      self.get_display_width(0) as uint, 
                                      self.get_display_height(0) as uint));
            }
            
            buf_as_slice(screen_ptr, num as uint, |x| {
                let mut result : Vec<Rectangle> = Vec::new();
                for &screen_info in x.iter() {
                    result.push(Rectangle(
                            screen_info.x_org as uint,
                            screen_info.y_org as uint,
                            screen_info.width as uint,
                            screen_info.height as uint));
                }
                result
            })
        }
    }

    fn get_number_of_screens(&self) -> uint {
        unsafe {
            XScreenCount(self.display) as uint
        }
    }

    fn get_display_width(&self, screen: uint) -> u32 {
        unsafe {
            XDisplayWidth(self.display, screen as i32) as u32
        }
    }

    fn get_display_height(&self, screen: uint) -> u32 {
        unsafe {
            XDisplayHeight(self.display, screen as i32) as u32
        }
    }

    fn get_window_name(&self, window: Window) -> String {
        if window == self.root { return String::from_str("root"); }
        unsafe {
            let mut name : *mut c_char = uninitialized();
            let name_ptr : *mut *mut c_char = &mut name;
            XFetchName(self.display, window, name_ptr);
            String::from_str(c_str_to_static_slice(transmute(*name_ptr)))
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        unsafe {
            let mut unused = 0u64;
            let mut children : *mut u64 = uninitialized();
            let children_ptr : *mut *mut u64 = &mut children;
            let mut num_children : c_uint = 0;
            XQueryTree(self.display, self.root, &mut unused, &mut unused, children_ptr, &mut num_children);
            let const_children : *const u64 = children as *const u64;
            buf_as_slice(const_children, num_children as uint, |x| {
                let mut result : Vec<Window> = Vec::new();
                for &child in x.iter() {
                    if child != self.root {
                        result.push(child);
                    }
                }
                result
            })
        }
    }

    fn set_window_border_width(&mut self, window: Window, border_width: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorderWidth(self.display, window, border_width as u32); 
        }
    }

    fn set_window_border_color(&mut self, window: Window, border_color: uint) {
        if window == self.root { return; }
        unsafe {
            XSetWindowBorder(self.display, window, border_color as Window);   
        }
    }

    fn resize_window(&mut self, window: Window, width: u32, height: u32) {
        unsafe {
            XResizeWindow(self.display, window, width, height);
        }
    }

    fn move_window(&mut self, window: Window, x: u32, y: u32) {
        unsafe {
            XMoveWindow(self.display, window, x as i32, y as i32);
        }
    }

    fn show_window(&mut self, window: Window) {
        unsafe {
            XMapWindow(self.display, window);
        }
    }

    fn event_pending(&self) -> bool {
        unsafe {
            XPending(self.display) != 0
        }
    }

    fn get_event(&mut self) -> WindowSystemEvent {
        unsafe {
            XSync(self.display, 0);
            XNextEvent(self.display, self.event);
        }
        
        let event_type : c_int = *self.get_event_as();

        match event_type as uint {
            MapRequest => {
                let event : &XMapRequestEvent = self.get_event_as();
                unsafe { XSelectInput(self.display, event.window, 0x000030); }
                WindowCreated(event.window)
            },
            EnterNotify => {
                let event : &XEnterWindowEvent = self.get_event_as();
                Enter(event.window) 
            },
            LeaveNotify => {
                let event : &XLeaveWindowEvent = self.get_event_as();
                Leave(event.window) 
            },
            _  => {
                debug!("Unknown event {}", event_type);
                UnknownEvent
            }
        }
    }
}