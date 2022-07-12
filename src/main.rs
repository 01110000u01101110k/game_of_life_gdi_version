use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use winapi::um::wingdi::RGB;
use windows::Win32::Devices::Display::COLORINFO;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::UI::WindowsAndMessaging::COLOR_WINDOW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};
use rand::{thread_rng, Rng};
#[derive(Debug, Copy, Clone)]
struct Cell {
    is_fill: bool,
    position_x: u32,
    position_y: u32,
}
#[derive(Debug)]
struct Cells {
    cells_array: Vec<Vec<Cell>>,
    change_event: bool,
}

const MAX_COLUMN_COUNT: u32 = 120;
const MAX_ROWS_COUNT: u32 = 60;

impl Cells {
    fn new() -> Self {
        Self {
            cells_array: Vec::new(),
            change_event: true,
        }
    }

    fn fill_cells_array(&mut self) {
        if self.cells_array.len() == 0 {
            let total_count = MAX_COLUMN_COUNT * MAX_ROWS_COUNT;
            let mut x: u32 = 1;
            let mut y: u32 = 1;

            let mut iter = 0;

            let mut is_fill: bool = false;

            let mut rand_rng = rand::thread_rng();

            self.cells_array.push(Vec::new());

            while iter != total_count {
                if rand_rng.gen_range(0..2) == 1 {
                    is_fill = true;
                } else {
                    is_fill = false;
                }

                self.cells_array[(y - 1) as usize].push(Cell {
                    is_fill: is_fill,
                    position_x: x,
                    position_y: y,
                });

                if x < MAX_COLUMN_COUNT {
                    x += 1;
                } else {
                    self.cells_array.push(Vec::new());
                    x = 1;
                    y += 1;
                }
                iter += 1;
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
    is_happened_change: bool,
}

impl GameState {
    fn new() -> Self {
        let mut new_game_state = Self {
            cells: Cells::new(),
            is_game_on: true,
            is_game_over: false,
            is_happened_change: true,
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
    rect: RECT,
    width: i32,
    height: i32,
    yellow_pen: HPEN,
    yellow_brush: HBRUSH,
    grey_pen: HPEN,
    grey_brush: HBRUSH,
}

impl WindowsApiState {
    fn new() -> Self {
        unsafe {
            let yellow_pen: HPEN = CreatePen(PS_SOLID, 1, RGB(223, 180, 13));
            let yellow_brush: HBRUSH = CreateSolidBrush(RGB(223, 180, 13));
            let grey_pen: HPEN = CreatePen(PS_SOLID, 1, RGB(51, 51, 51));
            let grey_brush: HBRUSH = CreateSolidBrush(RGB(51, 51, 51));

            Self {
                hwnd: HWND::default(),
                rect: RECT::default(),
                width: 0,
                height: 0,
                yellow_pen: yellow_pen,
                yellow_brush: yellow_brush,
                grey_pen: grey_pen,
                grey_brush: grey_brush,
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

fn draw_cell(
    paint_handle: HDC,
    window_state_info: &WindowsApiState,
    pen: HPEN,
    brush: HBRUSH,
    left_position: i32,
    top_position: i32,
    right_position: i32,
    bottom_position: i32,
) {
    unsafe {
        let pen = SelectObject(paint_handle, pen);
        let brush = SelectObject(paint_handle, brush);

        RoundRect(
            paint_handle,
            left_position,
            top_position,
            right_position,
            bottom_position,
            2,
            2,
        );

        /*DeleteObject(pen);
        DeleteObject(brush);*/
    }
}

fn cell_status_update() {
    let cells_arr_copy = GAME_STATE.lock().unwrap().cells.cells_array.clone();
    let mut new_cells_array = GAME_STATE.lock().unwrap().cells.cells_array.clone();

    for cell_column in cells_arr_copy.iter() {
        for cell in cell_column {
            let mut near_cells: Vec<Cell> = Vec::new();

            let cell_position_x: i32 = (cell.position_x - 1) as i32;
            let cell_position_y: i32 = (cell.position_y - 1) as i32;

            if cell_position_x - 1 > -1 && cell_position_y - 1 > -1 {
                near_cells.push(cells_arr_copy[(cell_position_y - 1) as usize][(cell_position_x - 1) as usize]);
            }

            if cell_position_y - 1 > -1 && cell_position_x < MAX_COLUMN_COUNT as i32 {
                near_cells.push(cells_arr_copy[(cell_position_y - 1) as usize][cell_position_x as usize]);
            }

            if cell_position_y - 1 > -1 && cell_position_x + 1 < MAX_COLUMN_COUNT as i32 {
                near_cells.push(cells_arr_copy[(cell_position_y - 1) as usize][(cell_position_x + 1) as usize]);
            }


            if cell_position_x - 1 > -1 && cell_position_y < MAX_ROWS_COUNT as i32 {
                near_cells.push(cells_arr_copy[cell_position_y as usize][(cell_position_x - 1) as usize]);
            }

            if cell_position_x + 1 < MAX_COLUMN_COUNT as i32 && cell_position_y < MAX_ROWS_COUNT as i32 {
                near_cells.push(cells_arr_copy[cell_position_y as usize][(cell_position_x + 1) as usize]);
            }

            
            if cell_position_x - 1 > -1 && cell_position_y + 1 < MAX_ROWS_COUNT as i32 {
                near_cells.push(cells_arr_copy[(cell_position_y + 1) as usize][(cell_position_x - 1) as usize]);
            }

            if cell_position_y + 1 < MAX_ROWS_COUNT as i32 && cell_position_x < MAX_COLUMN_COUNT as i32 {
                near_cells.push(cells_arr_copy[(cell_position_y + 1) as usize][cell_position_x as usize]);
            }

            if cell_position_x + 1 < MAX_COLUMN_COUNT as i32 && cell_position_y + 1 < MAX_ROWS_COUNT as i32 {
                near_cells.push(cells_arr_copy[(cell_position_y + 1) as usize][(cell_position_x + 1) as usize]);
            }

            let mut count_near_cells = 0;

            for curent_near_cell in near_cells {
                if curent_near_cell.is_fill {
                    count_near_cells += 1;
                }
            }

            if !cell.is_fill && count_near_cells == 3 {
                new_cells_array[cell_position_y as usize][cell_position_x as usize].is_fill = true;
            } else if cell.is_fill && (count_near_cells < 2 || count_near_cells > 3) {
                new_cells_array[cell_position_y as usize][cell_position_x as usize].is_fill = false;
            }
        }
    }

    GAME_STATE.lock().unwrap().cells.cells_array = new_cells_array;
}

fn draw_cells(begin_paint: HDC, window_state_info: &WindowsApiState) {
    let size: i32 = 14;
    let mut left_position: i32 = size;
    let mut top_position: i32 = size;
    let mut right_position: i32 = size * 2;
    let mut bottom_position: i32 = size * 2;

    let mut prev_cell_position_y: u32 = 1;

    for cell_column in GAME_STATE.lock().unwrap().cells.cells_array.iter() {
        for cell in cell_column {
            if prev_cell_position_y < cell.position_y {
                left_position = size;
                top_position = top_position + size;
                right_position = size * 2;
                bottom_position = bottom_position + size;
            }

            if cell.is_fill {
                draw_cell(
                    begin_paint,
                    &window_state_info,
                    window_state_info.yellow_pen,
                    window_state_info.yellow_brush,
                    left_position,
                    top_position,
                    right_position,
                    bottom_position,
                );
            } else {
                draw_cell(
                    begin_paint,
                    &window_state_info,
                    window_state_info.grey_pen,
                    window_state_info.grey_brush,
                    left_position,
                    top_position,
                    right_position,
                    bottom_position,
                );
            }

            left_position = left_position + size;
            right_position = right_position + size;

            prev_cell_position_y = cell.position_y;
        }
    }
}

fn draw() {
    unsafe {
        let window_state_info = WINDOW_STATE_INFO.lock().unwrap();
        let mut paint_struct = PAINTSTRUCT::default();
        let begin_paint = BeginPaint(window_state_info.hwnd, &mut paint_struct);

        draw_cells(begin_paint, &window_state_info);

        EndPaint(window_state_info.hwnd, &mut paint_struct);
    }
}

fn check_rules_and_draw() {
    cell_status_update();
    draw();
}

fn start_game_loop() {
    while GAME_STATE.lock().unwrap().is_game_on {
        unsafe{
            let window_state = WINDOW_STATE_INFO.lock().unwrap();
            //RedrawWindow(window_state.hwnd, &window_state.rect, None, RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ALLCHILDREN);
            InvalidateRect(window_state.hwnd, &window_state.rect, false);
        }
        thread::sleep(Duration::from_millis(100)); // ограничиваю скорость обновления цикла игры
    }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = "window";

        let background: HBRUSH = CreateSolidBrush(RGB(28, 28, 28));

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: PCSTR(b"window\0".as_ptr()),
            hbrBackground: background,

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
            std::ptr::null(),
        );

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

                let mut rect = RECT::default();
                let window_size = GetClientRect(window, &mut rect);

                WINDOW_STATE_INFO.lock().unwrap().rect = rect;
                WINDOW_STATE_INFO.lock().unwrap().width = rect.right;
                WINDOW_STATE_INFO.lock().unwrap().height = rect.bottom;

                thread::spawn(|| {
                    // запускаю поток, для работы игрового цикла, что-бы не блокировать цикл обработки событий окна, при долгой обработке игровой логики
                    start_game_loop(); // запускаю игровой цикл
                });

                LRESULT(0)
            }
            WM_CLOSE => {
                if MessageBoxW(
                    window,
                    "Вы хотите закрыть приложение?",
                    APP_NAME,
                    MB_OKCANCEL,
                ) == IDOK
                {
                    DestroyWindow(window);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                let mut rect = WINDOW_STATE_INFO.lock().unwrap().rect;
                let window_size = GetClientRect(window, &mut rect);

                WINDOW_STATE_INFO.lock().unwrap().rect = rect;
                WINDOW_STATE_INFO.lock().unwrap().width = rect.right;
                WINDOW_STATE_INFO.lock().unwrap().height = rect.bottom;

                LRESULT(0)
            }
            WM_PAINT => {
                check_rules_and_draw();

                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
