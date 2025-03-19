use iced::widget::{ button, column, container, text, scrollable };
use iced::{ executor, Application, Command, Element, Settings, Theme };
use iced::Color;
use std::process::Command as ProcessCommand;

fn list_wifi_networks() -> Vec<(String, String, i32)> {
    let mut networks = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let output = ProcessCommand::new("nmcli")
            .args(["-t", "-f", "SSID,BSSID,SIGNAL", "dev", "wifi"])
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let wifi_list = String::from_utf8_lossy(&output.stdout);
            for wifi in wifi_list.lines() {
                if let Some((mac_address_and_ssid, signal_strength)) = wifi.rsplit_once(":") {
                    if let Ok(strength) = signal_strength.parse::<i32>() {
                        let mut parts = mac_address_and_ssid.splitn(2, ":");
                        if let Some(name) = parts.next() {
                            if let Some(mac_address) = parts.next() {
                                if !name.trim().is_empty() && !mac_address.trim().is_empty() {
                                    networks.push((
                                        name.to_string(),
                                        mac_address.to_string(),
                                        strength,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    networks
}

fn signal_color(strength: i32) -> Color {
    match strength {
        0..=20 => Color::from_rgb8(139, 0, 0),
        21..=50 => Color::from_rgb8(255, 165, 0),
        51..=80 => Color::from_rgb8(255, 255, 0),
        81..=100 => Color::from_rgb8(0, 255, 0),
        _ => Color::BLACK,
    }
}

pub fn main() -> iced::Result {
    WirelessScanner::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    Scan,
    ScanResult(Vec<(String, String, i32)>),
}

struct WirelessScanner {
    networks: Vec<(String, String, i32)>,
    scanning: bool,
}

impl Default for WirelessScanner {
    fn default() -> Self {
        Self { networks: vec![], scanning: false }
    }
}

impl Application for WirelessScanner {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Wireless Scanner")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Scan => {
                if self.scanning {
                    return Command::none();
                }
                self.scanning = true;
                self.networks = vec![];
                Command::perform(async { list_wifi_networks() }, Message::ScanResult)
            }
            Message::ScanResult(results) => {
                self.networks = results;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let scan_button = button("Scan").on_press(Message::Scan);

        let network_list = self.networks
            .iter()
            .fold(column![], |col, (ssid, bssid, strength)| {
                col.push(
                    text(
                        format!("SSID: {} | BSSID: {} | Strength: {}%", ssid, bssid, strength)
                    ).style(iced::theme::Text::Color(signal_color(*strength)))
                )
            });

        let scrollable_network_list = scrollable(network_list).height(iced::Length::Fill);

        container(column![scan_button, scrollable_network_list]).center_x().center_y().into()
    }
}
