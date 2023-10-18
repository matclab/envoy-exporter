use anyhow::Result;
use log;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct IniConfig {
    pub listen_port: Option<u32>,
    pub systems: Vec<IniSystem>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub listen_port: Option<u32>,
    pub systems: Vec<System>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct IniSystem {
    pub host: String,
    pub url: String,
    pub user: String,
    pub pass: String,
    pub sn: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub host: String,
    pub url: String,
    pub user: String,
    pub pass: String,
    pub sn: String,
    pub token: String,
}

impl IniConfig {
    pub fn from_file(file: &str) -> Result<IniConfig> {
        let mut f = File::open(file)?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        let config: IniConfig = toml::from_str(&s)?;
        log::debug!("config {:?}", config);
        Ok(config)
    }
}

impl Config {
    pub fn from_file(file: &str) -> Result<Config> {
        let mut iniconfig: IniConfig = IniConfig::from_file(file)?;
        let mut systems: Vec<System> = vec![];
        for sys in &mut iniconfig.systems {
            let token = sys.clone().get_token();
            systems.push(System {
                host: sys.host.clone(),
                url: sys.url.clone(),
                user: sys.user.clone(),
                pass: sys.pass.clone(),
                sn: sys.sn.clone(),
                token,
            })
        }
        Ok(Config {
            listen_port: iniconfig.listen_port,
            systems,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenData {
    session_id: String,
    serial_num: String,
    username: String,
}

#[derive(Deserialize)]
struct SessionMessage {
    session_id: String,
}

impl IniSystem {
    fn get_token_when_logged(self, resp: ureq::Response) -> String {
        let json: Result<SessionMessage> = resp.into_json().map_err(anyhow::Error::from);

        if let Ok(session) = json {
            let data = TokenData {
                session_id: session.session_id,
                serial_num: self.sn,
                username: self.user.clone(),
            };

            //data = {'session_id': response_data['session_id'], 'serial_num': envoy_serial, 'username':
            //response = requests.post('https://entrez.enphaseenergy.com/tokens', json=data)
            //token_raw = response.text
            match ureq::post("https://entrez.enphaseenergy.com/tokens").send_json(
                serde_json::to_value(data).unwrap_or_else(|e| {
                    panic!(
                        "serialization of user '{:?}' failed with {:?}",
                        &self.user, &e
                    )
                }),
            ) {
                Ok(resp) => return resp.into_string().unwrap_or("".to_string()),
                Err(ureq::Error::Transport(t)) => {
                    log::error!("Error {}", t);
                }
                Err(ureq::Error::Status(code, response)) => {
                    log::error!("Error {} : {:?}", code, response);
                }
            }
        } else {
            log::error!(
                "Unable to deserialize session from enphase response" // TODO, unable to display response : see https://github.com/algesten/ureq/issues/505
            );
        }
        return "".to_string();
    }
    pub fn get_token(self) -> String {
        //data = {'user[email]': user, 'user[password]': password}
        //response = requests.post('https://enlighten.enphaseenergy.com/login/login.json?',
        //data=data) response_data = json.loads(response.text)
        //user}
        let user = User {
            email: self.user.clone(),
            password: self.pass.clone(),
        };
        match ureq::post("https://enlighten.enphaseenergy.com/login/login.json?").send_json(
            serde_json::to_value(user).unwrap_or_else(|e| {
                panic!(
                    "serialization of user '{:?}' failed with {:?}",
                    &self.user, &e
                )
            }),
        ) {
            Ok(resp) => {
                self.get_token_when_logged(resp);
            }
            Err(ureq::Error::Transport(t)) => {
                log::error!("Error {}", t);
            }
            Err(ureq::Error::Status(code, response)) => {
                log::error!("Error {} : {:?}", code, response);
            }
        }
        "".to_string()
    }
}
