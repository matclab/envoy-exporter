use anyhow::{Result,Context};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize, Clone)]
pub struct IniConfig {
    pub listen_port: Option<u32>,
    pub systems: Vec<IniSystem>,
}
#[derive(Debug, Clone)]
pub struct Config {
    pub listen_port: Option<u32>,
    pub systems: Vec<System>,
    pub agent: ureq::Agent,
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
    pub fn from_file(file: &str, agent: ureq::Agent) -> Result<Config> {
        let mut iniconfig: IniConfig = IniConfig::from_file(file)?;
        let mut systems: Vec<System> = vec![];
        for sys in &mut iniconfig.systems {
            let token = sys.clone().get_token();
            log::debug!("token : {}", token);
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
            agent
        })
    }
}


impl IniSystem {
    fn get_token_when_logged(self, resp: ureq::Response) -> Result<String> {
        println!("login resp{:?}", resp);
        let json: serde_json::Value = resp.into_json()?;
        let session_id = json["session_id"].as_str().with_context(|| format!("session_id not found in {:?}", json))?;

        let data = ureq::json!( {
            "session_id": session_id,
            "serial_num": self.sn,
            "username": self.user.clone(),
        });

        //data = {'session_id': response_data['session_id'], 'serial_num': envoy_serial, 'username':
        //response = requests.post('https://entrez.enphaseenergy.com/tokens', json=data)
        //token_raw = response.text
       Ok(ureq::post("https://entrez.enphaseenergy.com/tokens").send_json(data).map(|v| v.into_string())??)
    }

    pub fn get_token(self) -> String {
        //data = {'user[email]': user, 'user[password]': password}
        //response = requests.post('https://enlighten.enphaseenergy.com/login/login.json?',
        //data=data) response_data = json.loads(response.text)
        //user}
        match ureq::post("https://enlighten.enphaseenergy.com/login/login.json?")
            .send_form(&[("user[email]", &self.user), ("user[password]", &self.pass)])
        {
            Ok(resp) => {
                self.get_token_when_logged(resp)
                    //.context("Getting auth token from entrez.enphaseenergy.com")
                    .unwrap_or("".to_string())
            }
            Err(ureq::Error::Transport(t)) => {
                log::error!("Error {}", t);
        "".to_string()
            }
            Err(ureq::Error::Status(code, response)) => {
                log::error!("Error {} : {:?}", code, response);
        "".to_string()
            }
        }
    }
}
