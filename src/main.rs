use windows::{core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect, Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW;
use windows::Win32::Graphics::Gdi::SelectObject;
use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;
use std::sync::Mutex;
use windows::Win32::Devices::Display::COLORINFO;
use winapi::um::wingdi::RGB;
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

struct Cell {
    position_x: u32,
    position_y: u32
}

struct Cells {
    cells_array: Vec<Cell>,
    change_event: bool
}

impl Cells {
    fn new() -> Self {
        Self {
            cells_array: Vec::new(),
            change_event: true
        }
    }

    fn fill_cells_array(&mut self) {
        if self.cells_array.len() == 0 {
            let total_count = 20;
            let mut x = 0;
            let mut y = 0;

            while x + y != total_count {
                if x < 10 {
                    x += 1;
                } else {
                    x = 0;
                    y += 1;
                }

                self.cells_array.push(Cell {
                    position_x: x,
                    position_y: y
                })
            }
        }
    }

    fn change_event_to_true(&mut self) {
        if !self.change_event {
            self.change_event = true;
        }
    }

    fn change_event_to_false(&mut self) {
        if self.change_event {
            self.change_event = false;
        }
    }
}

struct GameState {
    cells: Cells,
    is_game_on: bool,
    is_game_over: bool,
    is_happened_change: bool
}

impl GameState {
    fn new() -> Self {
        let mut new_game_state = Self {
            cells: Cells::new(),
            is_game_on: true,
            is_game_over: false,
            is_happened_change: true
        };
        
        new_game_state.cells.fill_cells_array();

        new_game_state
    }

    fn change_game_state(&mut self) {
        self.is_game_on = !self.is_game_on;
    }

    fn change_game_over_state(&mut self) {
        self.is_game_over = !self.is_game_over;
    }
}

struct WindowsApiState {
    hwnd: HWND,
    width: i32,
    height: i32,
    yellow_pen: HPEN,
    yellow_brush: HBRUSH
}

impl WindowsApiState {
    fn new() -> Self {
        unsafe {
            Self {
                hwnd: HWND::default(),
                width: 0,
                height: 0,
                yellow_pen: CreatePen(PS_SOLID, 1, RGB(223, 180, 13)),
                yellow_brush: CreateSolidBrush(RGB(223, 180, 13))
            }
        }
    }
    fn change_hwnd(&mut self, new_hwnd: HWND) {
        self.hwnd = new_hwnd;
    }
}

const APP_NAME: &str = "Game of life";

lazy_static! {
    static ref GAME_STATE: Mutex<GameState> = {
        let data = GameState::new();
        Mutex::new(data)
    };
    static ref WINDOW_STATE_INFO: Mutex<WindowsApiState> = {
        let mut data = WindowsApiState::new();
        Mutex::new(data)
    };
}

fn draw_cell(paint_handle: HDC, window_state_info: &WindowsApiState, left_position: i32, top_position: i32, right_position: i32, bottom_position: i32) {
    unsafe {
        SelectObject(paint_handle, window_state_info.yellow_pen);
        SelectObject(paint_handle, window_state_info.yellow_brush);

        RoundRect(
            paint_handle,
            left_position,
            top_position,
            right_position,
            bottom_position,
            2,
            2
        );
    }
}

fn fill_color_client_window_rect(paint_handle: HDC, paint_struct: PAINTSTRUCT, color: HBRUSH) {
    unsafe {
        FillRect(paint_handle, &paint_struct.rcPaint, color);
    }
}

fn draw_cells() {
    unsafe {
        let size: i32 = 10;
        let mut left_position: i32 = size;
        let mut top_position: i32 = size;
        let mut right_position: i32 = size * 2;
        let mut bottom_position: i32 = size * 2;
        
        let window_state_info = WINDOW_STATE_INFO.lock().unwrap();
        let mut paint_struct = PAINTSTRUCT::default();
        let begin_paint = BeginPaint(window_state_info.hwnd, &mut paint_struct);

        for cell in GAME_STATE.lock().unwrap().cells.cells_array.iter() {
            println!("left_position {}", left_position);
            fill_color_client_window_rect(begin_paint, paint_struct, HBRUSH((COLOR_WINDOW.0 + 1) as isize));
            draw_cell(begin_paint, &window_state_info, left_position, top_position, right_position, bottom_position);
            left_position = left_position + size;
            //top_position = top_position + size;
            right_position = top_position + size;
            //bottom_position = top_position + size;
        }

        EndPaint(window_state_info.hwnd, &mut paint_struct);
    }
}

fn start_game_loop() {
    println!("{}", GAME_STATE.lock().unwrap().is_game_on);

    while GAME_STATE.lock().unwrap().is_game_on {
        GAME_STATE.lock().unwrap().change_game_over_state();

        draw_cells();
        println!("some game logic {:?}", WINDOW_STATE_INFO.lock().unwrap().hwnd);
        thread::sleep(Duration::from_millis(1000)); // ограничиваю скорость обновления цикла игры
    }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = "window";

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: PCSTR(b"window\0".as_ptr()),

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            Default::default(), 
            window_class, 
            APP_NAME, 
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, 
            CW_USEDEFAULT, 
            CW_USEDEFAULT, 
            CW_USEDEFAULT, 
            CW_USEDEFAULT, 
            None, 
            None, 
            instance, 
            std::ptr::null()
        );

        thread::spawn(|| { // запускаю поток, для работы игрового цикла, что-бы не блокировать цикл обработки событий окна, при долгой обработке игровой логики
            start_game_loop(); // запускаю игровой цикл
        });

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_CREATE => {
                WINDOW_STATE_INFO.lock().unwrap().change_hwnd(window);

                LRESULT(0)
            }
            WM_CLOSE => {
                if MessageBoxW(window, "Вы хотите закрыть приложение?", APP_NAME, MB_OKCANCEL) == IDOK {
                    DestroyWindow(window);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT");

                let mut rect = RECT::default();
                let window_size = GetClientRect(window, &mut rect);

                WINDOW_STATE_INFO.lock().unwrap().width = rect.right;
                WINDOW_STATE_INFO.lock().unwrap().height = rect.bottom;

                draw_cells();

                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}