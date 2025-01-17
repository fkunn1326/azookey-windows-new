use anyhow::Context as _;
use protos::proto::window_service_server::{
    WindowService as WindowServiceProto, WindowServiceServer,
};
use protos::proto::{EmptyResponse, SetPositionRequest};
use tao::dpi::{PhysicalPosition, PhysicalSize};
use tao::platform::windows::{
    EventLoopBuilderExtWindows, WindowBuilderExtWindows, WindowExtWindows,
};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use tokio::sync::mpsc;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        SetWindowLongW, ShowWindow, GWL_EXSTYLE, GWL_STYLE, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    },
};
use wry::WebViewBuilder;

#[derive(Debug, Clone)]
struct WindowController {
    sender: mpsc::Sender<WindowAction>,
}

impl WindowController {
    fn new(sender: mpsc::Sender<WindowAction>) -> Self {
        Self { sender }
    }
}

// ウィンドウ操作コマンド
#[derive(Debug)]
enum WindowAction {
    Show,
    Hide,
    SetPosition { x: i32, y: i32 },
}

#[derive(Debug)]
struct WindowService {
    controller: WindowController,
}

#[tonic::async_trait]
impl WindowServiceProto for WindowService {
    async fn show_window(
        &self,
        _request: Request<EmptyResponse>,
    ) -> Result<Response<EmptyResponse>, Status> {
        self.controller
            .sender
            .send(WindowAction::Show)
            .await
            .unwrap();
        Ok(Response::new(EmptyResponse {}))
    }

    async fn hide_window(
        &self,
        _request: Request<EmptyResponse>,
    ) -> Result<Response<EmptyResponse>, Status> {
        self.controller
            .sender
            .send(WindowAction::Hide)
            .await
            .unwrap();
        Ok(Response::new(EmptyResponse {}))
    }
    async fn set_window_position(
        &self,
        request: Request<SetPositionRequest>,
    ) -> Result<Response<EmptyResponse>, Status> {
        let position = request.into_inner().position.unwrap();
        let x = position.x;
        let y = position.y;
        self.controller
            .sender
            .send(WindowAction::SetPosition { x, y })
            .await
            .unwrap();

        Ok(Response::new(EmptyResponse {}))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let event_loop = EventLoopBuilder::<String>::with_user_event()
        .with_any_thread(true)
        .build();
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_title("CandidateList")
        .with_focused(false)
        .with_visible(false)
        .with_undecorated_shadow(false)
        .with_transparent(true)
        .build(&event_loop)
        .context("Failed to create window")?;

    // set size
    window.set_inner_size(PhysicalSize::new(275.0, 275.0));

    // set z-order
    window.set_always_on_top(true);

    let hwnd = window.hwnd() as *mut std::ffi::c_void;

    // set extended window style
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
    unsafe {
        let exnewstyle = WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_TOPMOST.0;
        SetWindowLongW(HWND(hwnd), GWL_EXSTYLE, exnewstyle as i32);

        let style = WS_POPUP.0;
        SetWindowLongW(HWND(hwnd), GWL_STYLE, style as i32);
    };

    let _webview = WebViewBuilder::new()
        .with_transparent(true)
        .with_html(
            r##"
            <html>
                <head>
                    <style>
                        body, html {
                            overscroll-behavior: none;
                        }
                        body {
                            margin: 0;
                            padding: 7px;
                            filter: drop-shadow(3px 3px 3px rgba(0, 0, 0, 0.1));
                        }
                        main {
                            width: 100%;
                            height: 100%;
                            padding: 8 8 30 8;
                            border: 1px solid #E4E4E4;
                            border-radius: 10px;
                            background-color: #FFFFFF;
                            box-sizing: border-box;
                        }
                        ol {
                            margin: 0;
                            padding: 0;
                            height: 100%;
                            overflow-y: scroll;
                            scroll-snap-type: y proximity;
                            list-style-position: inside;
                            list-style-type: none;
                            counter-reset: number 0;
                            user-select: none;
                            cursor: pointer;

                            &::-webkit-scrollbar {
                                width: 5px;
                            }

                            &::-webkit-scrollbar-thumb {
                                background-color: #BCBCBC;
                                border-radius: 10px;
                            }
                        }
                        li {
                            padding: 0.5rem;
                            font-size: 0.9rem;
                            display: flex;
                            align-items: center;
                            scroll-snap-align: start;

                            &::before {
                                content: counter(number);
                                counter-increment: number 1;
                                color: #636363;
                                font-weight: bold;
                                font-size: 0.75rem;
                                margin: 0 0.75rem 0 2;
                            }

                            &:hover {
                                background-color: #D4F0FF;
                                border-radius: 3px;
                                margin-right: 5px;
                                outline: 1px solid #2CB5FF;
                                outline-offset: -1px;
                            }
                        }
                        footer {
                            display: flex;
                            justify-content: space-between;
                            align-items: center;
                            padding: 8 10 5 10;
                            border-top: 1px solid #E4E4E4;
                            font-size: 0.8rem;
                            user-select: none;
                        }
                    </style>
                </head>
                <body style="margin: 0;">
                    <main>
                        <ol>
                            <li>変換テキスト1</li>
                            <li>変換テキスト2</li>
                            <li>変換テキスト3</li>
                            <li>変換テキスト4</li>
                            <li>変換テキスト5</li>
                            <li>変換テキスト6</li>
                            <li>変換テキスト7</li>
                            <li>変換テキスト8</li>
                            <li>変換テキスト9</li>
                        </ol>
                        <footer>
                            <svg width="20" height="14" viewBox="0 0 22 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                                <path d="M3.5 8C4.59202 9.04403 7.54398 10.3978 13.5068 9.93754M1.25349 5.39919C2.77722 0.413397 8.08911 0.79692 10.9673 1.24436C14.2687 1.71311 20.8969 3.82675 20.9985 8.53129C21.1255 14.412 13.1894 15.3069 10.0784 14.9233C6.96748 14.5398 -0.46071 13.0696 1.25349 5.39919Z" stroke="#838384" stroke-width="1.5" stroke-linecap="round"/>
                            </svg>
                        </footer>
                    </main>
                </body>
            </html>"##,
        )
        .build(&window)
        .context("Failed to create webview")?;

    window.set_outer_position(PhysicalPosition::new(500f64, 500f64));

    // initialize window controller
    let (tx, mut rx) = mpsc::channel(32);
    let window_controller = WindowController::new(tx.clone());
    let grpc_service = WindowService {
        controller: window_controller.clone(),
    };

    // start grpc server
    tokio::spawn(async move {
        let addr = "[::1]:50052".parse().unwrap();

        println!("WindowServer listening on {}", addr);
        Server::builder()
            .add_service(WindowServiceServer::new(grpc_service))
            .serve(addr)
            .await
            .expect("gRPC server failed");
    });

    // handle window actions
    tokio::spawn(async move {
        while let Some(action) = rx.recv().await {
            match action {
                WindowAction::Show => {
                    let _ = unsafe {
                        ShowWindow(
                            HWND(window.hwnd() as *mut std::ffi::c_void),
                            SW_SHOWNOACTIVATE,
                        )
                    };
                }
                WindowAction::Hide => {
                    let _ = unsafe {
                        ShowWindow(HWND(window.hwnd() as *mut std::ffi::c_void), SW_HIDE)
                    };
                }
                WindowAction::SetPosition { x, y } => {
                    window.set_outer_position(PhysicalPosition::new(x as f64, y as f64));
                }
            }
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
