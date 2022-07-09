use windows::{core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect, Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW;
use std::thread;
use std::time::Duration;

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
    is_game_over: bool
}

impl GameState {
    fn new() -> Self {
        let mut new_game_state = Self {
            cells: Cells::new(),
            is_game_on: false,
            is_game_over: false
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

const APP_NAME: &str = "Game of life";

fn start_game_loop() {
    'game_loop: loop {
        /*let mut game_state = GameState::new();

        if game_state.is_game_on {
            
        }*/

        println!("some game logic");
        thread::sleep(Duration::from_millis(1000));
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
            WM_CLOSE => {
                if MessageBoxW(window, "Вы хотите закрыть приложение?", APP_NAME, MB_OKCANCEL) == IDOK {
                    DestroyWindow(window);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                println!("WM_PAINT");
                //ValidateRect(window, std::ptr::null());
                let mut paint_struct = PAINTSTRUCT::default();
                let begin_paint = BeginPaint(window, &mut paint_struct);

                // All painting occurs here, between BeginPaint and EndPaint.

                FillRect(begin_paint, &paint_struct.rcPaint, HBRUSH((COLOR_WINDOW.0 + 1) as isize) );

                EndPaint(window, &paint_struct);
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