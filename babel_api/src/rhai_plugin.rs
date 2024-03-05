use crate::plugin_config::Service;
use crate::{
    engine::{self, Engine, HttpResponse, JobConfig, JobStatus, JrpcRequest, ShResponse},
    metadata::{check_metadata, BlockchainMetadata},
    plugin::{ApplicationStatus, Plugin, StakingStatus, SyncStatus},
    plugin_config::{self, PluginConfig},
};
use eyre::{anyhow, bail, Context, Error, Result};
use rhai::{
    self,
    serde::{from_dynamic, to_dynamic},
    Dynamic, FnPtr, Map, AST,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};
use tracing::Level;

const DOWNLOAD_JOB_NAME: &str = "download";
const UPLOAD_JOB_NAME: &str = "upload";
const PLUGIN_CONFIG_CONST_NAME: &str = "PLUGIN_CONFIG";
const BABEL_VERSION_CONST_NAME: &str = "BABEL_VERSION";

#[derive(Debug)]
pub struct RhaiPlugin<E> {
    babel_engine: Arc<E>,
    ast: AST,
    rhai_engine: rhai::Engine,
}

impl<E: Engine + Sync + Send + 'static> Clone for RhaiPlugin<E> {
    fn clone(&self) -> Self {
        let mut clone = Self {
            babel_engine: self.babel_engine.clone(),
            rhai_engine: rhai::Engine::new(),
            ast: self.ast.clone(),
        };
        clone.init_rhai_engine();
        clone
    }
}

pub fn read_metadata(script: &str) -> Result<BlockchainMetadata> {
    find_metadata_in_ast(
        &new_rhai_engine()
            .compile(script)
            .with_context(|| "Rhai syntax error")?,
    )
}

fn new_rhai_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_max_expr_depths(64, 32);
    engine
}

fn find_metadata_in_ast(ast: &AST) -> Result<BlockchainMetadata> {
    let meta = find_const_in_ast(ast, "METADATA")?;
    check_metadata(&meta)?;
    Ok(meta)
}

fn find_const_in_ast<T: for<'a> Deserialize<'a>>(ast: &AST, name: &str) -> Result<T> {
    let mut vars = ast.iter_literal_variables(true, false);
    if let Some((_, _, dynamic)) = vars.find(|(v, _, _)| *v == name) {
        let value: T = from_dynamic(&dynamic)
            .with_context(|| format!("Invalid Rhai script - failed to deserialize {name}"))?;
        Ok(value)
    } else {
        Err(anyhow!("Invalid Rhai script - missing {name} constant"))
    }
}

impl<E: Engine + Sync + Send + 'static> RhaiPlugin<E> {
    pub fn new(script: &str, babel_engine: E) -> Result<Self> {
        let rhai_engine = new_rhai_engine();
        // compile script to AST
        let ast = rhai_engine
            .compile(script)
            .with_context(|| "Rhai syntax error")?;
        let mut plugin = RhaiPlugin {
            babel_engine: Arc::new(babel_engine),
            rhai_engine,
            ast,
        };
        plugin.check_babel_version()?;
        plugin.init_rhai_engine();
        Ok(plugin)
    }

    fn check_babel_version(&self) -> Result<()> {
        if let Ok(min_babel_version) =
            find_const_in_ast::<String>(&self.ast, BABEL_VERSION_CONST_NAME)
        {
            let min_babel_version = semver::Version::parse(&min_babel_version)?;
            let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
            if version < min_babel_version {
                bail!(
                "Required minimum babel version is `{min_babel_version}`, running is `{version}`"
            );
            }
        }
        Ok(())
    }

    /// register all Babel engine methods
    fn init_rhai_engine(&mut self) {
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("create_job", move |job_name: &str, job_config: Dynamic| {
                into_rhai_result(babel_engine.create_job(job_name, from_dynamic(&job_config)?))
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("start_job", move |job_name: &str| {
                into_rhai_result(babel_engine.start_job(job_name))
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("start_job", move |job_name: &str, job_config: Dynamic| {
                into_rhai_result(
                    babel_engine
                        .create_job(job_name, from_dynamic(&job_config)?)
                        .and_then(|_| babel_engine.start_job(job_name)),
                )
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("stop_job", move |job_name: &str| {
                into_rhai_result(babel_engine.stop_job(job_name))
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("job_info", move |job_name: &str| {
                to_dynamic(into_rhai_result(babel_engine.job_info(job_name))?)
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("get_jobs", move || {
            to_dynamic(into_rhai_result(babel_engine.get_jobs())?)
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("run_jrpc", move |req: Dynamic, timeout: i64| {
                let timeout = into_rhai_result(timeout.try_into().map_err(Error::new))?;
                let req = into_rhai_result(from_dynamic::<BareJrpcRequest>(&req)?.try_into())?;
                to_dynamic(into_rhai_result(
                    babel_engine
                        .run_jrpc(req, Some(Duration::from_secs(timeout)))
                        .map(DressedHttpResponse::from),
                )?)
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("run_jrpc", move |req: Dynamic| {
                let req = into_rhai_result(from_dynamic::<BareJrpcRequest>(&req)?.try_into())?;
                to_dynamic(into_rhai_result(
                    babel_engine
                        .run_jrpc(req, None)
                        .map(DressedHttpResponse::from),
                )?)
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("run_rest", move |req: Dynamic, timeout: i64| {
                let timeout = into_rhai_result(timeout.try_into().map_err(Error::new))?;
                to_dynamic(into_rhai_result(
                    babel_engine
                        .run_rest(from_dynamic(&req)?, Some(Duration::from_secs(timeout)))
                        .map(DressedHttpResponse::from),
                )?)
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("run_rest", move |req: Dynamic| {
                to_dynamic(into_rhai_result(
                    babel_engine
                        .run_rest(from_dynamic(&req)?, None)
                        .map(DressedHttpResponse::from),
                )?)
            });
        self.rhai_engine
            .register_type_with_name::<DressedHttpResponse>("HttpResponse")
            .register_fn("expect", DressedHttpResponse::expect_with)
            .register_fn("expect", DressedHttpResponse::expect);
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("run_sh", move |body: &str, timeout: i64| {
                let timeout = into_rhai_result(timeout.try_into().map_err(Error::new))?;
                to_dynamic(into_rhai_result(
                    babel_engine
                        .run_sh(body, Some(Duration::from_secs(timeout)))
                        .map(DressedShResponse::from),
                )?)
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("run_sh", move |body: &str| {
            to_dynamic(into_rhai_result(
                babel_engine.run_sh(body, None).map(DressedShResponse::from),
            )?)
        });
        self.rhai_engine
            .register_type_with_name::<DressedShResponse>("ShResponse")
            .register_fn("unwrap", DressedShResponse::unwrap);
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("sanitize_sh_param", move |param: &str| {
                into_rhai_result(babel_engine.sanitize_sh_param(param))
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn(
            "render_template",
            move |template: &str, output: &str, params: Map| {
                into_rhai_result(babel_engine.render_template(
                    Path::new(&template.to_string()),
                    Path::new(&output.to_string()),
                    &rhai::format_map_as_json(&params),
                ))
            },
        );
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("node_params", move || {
            Map::from_iter(
                babel_engine
                    .node_params()
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into())),
            )
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine
            .register_fn("save_data", move |value: &str| {
                into_rhai_result(babel_engine.save_data(value))
            });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("load_data", move || {
            into_rhai_result(babel_engine.load_data())
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.on_debug(move |msg, _, _| {
            babel_engine.log(Level::DEBUG, msg);
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("info", move |msg: &str| {
            babel_engine.log(Level::INFO, msg);
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("warn", move |msg: &str| {
            babel_engine.log(Level::WARN, msg);
        });
        let babel_engine = self.babel_engine.clone();
        self.rhai_engine.register_fn("error", move |msg: &str| {
            babel_engine.log(Level::ERROR, msg);
        });

        // register other utils
        self.rhai_engine.register_fn("parse_json", |json: &str| {
            rhai::Engine::new().parse_json(json, true)
        });
        self.rhai_engine.register_fn("parse_hex", |hex: &str| {
            i64::from_str_radix(hex.strip_prefix("0x").unwrap_or(hex), 16)
        });
    }

    fn call_fn<P: rhai::FuncArgs, R: Clone + Send + Sync + 'static>(
        &self,
        name: &str,
        args: P,
    ) -> Result<R> {
        let mut scope = rhai::Scope::new();
        scope.push("DATA_DRIVE_MOUNT_POINT", engine::DATA_DRIVE_MOUNT_POINT);
        scope.push(
            "BLOCKCHAIN_DATA_PATH",
            engine::BLOCKCHAIN_DATA_PATH.to_string_lossy(),
        );
        self.rhai_engine
            .call_fn::<R>(&mut scope, &self.ast, name, args)
            .with_context(|| format!("Rhai function '{name}' returned error"))
    }

    fn start_services(&self, services: Vec<Service>, needs: Vec<String>) -> Result<()> {
        for service in services {
            let name = service.name.clone();
            self.create_and_start_job(
                &name,
                plugin_config::build_service_job_config(service, needs.clone()),
            )?;
        }
        Ok(())
    }

    fn create_and_start_job(&self, name: &str, config: JobConfig) -> Result<()> {
        self.babel_engine
            .create_job(name, config)
            .and_then(|_| self.babel_engine.start_job(name))
    }
}

impl<E: Engine + Sync + Send + 'static> Plugin for RhaiPlugin<E> {
    fn metadata(&self) -> Result<BlockchainMetadata> {
        find_metadata_in_ast(&self.ast)
    }

    fn capabilities(&self) -> Vec<String> {
        self.ast
            .iter_functions()
            .map(|meta| meta.name.to_string())
            .collect()
    }

    fn init(&self) -> Result<()> {
        if let Some(init_meta) = self.ast.iter_functions().find(|meta| meta.name == "init") {
            if init_meta.params.is_empty() {
                self.call_fn(init_meta.name, ())
            } else {
                self.call_fn(init_meta.name, (Map::default(),))
            }
        } else {
            let config: PluginConfig = find_const_in_ast(&self.ast, PLUGIN_CONFIG_CONST_NAME)?;
            let mut init_jobs = vec![];
            if let Some(init_config) = config.init {
                for command in init_config.commands {
                    let resp = self.babel_engine.run_sh(&command, None)?;
                    if resp.exit_code != 0 {
                        bail!("init command '{command}' failed with exit_code: {resp:?}")
                    }
                }
                for job in init_config.jobs {
                    let name = job.name.clone();
                    self.create_and_start_job(&name, plugin_config::build_init_job_config(job))?;
                    init_jobs.push(name);
                }
            }
            if let Err(err) = self.create_and_start_job(
                DOWNLOAD_JOB_NAME,
                plugin_config::build_download_job_config(config.download, init_jobs.clone()),
            ) {
                if let Some(alternative_download) = config.alternative_download {
                    self.create_and_start_job(
                        DOWNLOAD_JOB_NAME,
                        plugin_config::build_alternative_download_job_config(
                            alternative_download,
                            init_jobs,
                        ),
                    )?;
                } else {
                    bail!("Download failed with no alternative provided: {err:#}");
                }
            }
            self.start_services(config.services, vec![DOWNLOAD_JOB_NAME.to_string()])?;
            Ok(())
        }
    }

    fn upload(&self) -> Result<()> {
        if self.ast.iter_functions().any(|meta| meta.name == "upload") {
            self.call_fn("upload", ())
        } else {
            let config: PluginConfig = find_const_in_ast(&self.ast, PLUGIN_CONFIG_CONST_NAME)?;
            for service in &config.services {
                self.babel_engine.stop_job(&service.name)?;
            }
            self.create_and_start_job(
                UPLOAD_JOB_NAME,
                plugin_config::build_upload_job_config(config.upload),
            )?;
            self.start_services(config.services, vec![UPLOAD_JOB_NAME.to_string()])?;
            Ok(())
        }
    }

    fn height(&self) -> Result<u64> {
        Ok(self.call_fn::<_, i64>("height", ())?.try_into()?)
    }

    fn block_age(&self) -> Result<u64> {
        Ok(self.call_fn::<_, i64>("block_age", ())?.try_into()?)
    }

    fn name(&self) -> Result<String> {
        self.call_fn("name", ())
    }

    fn address(&self) -> Result<String> {
        self.call_fn("address", ())
    }

    fn consensus(&self) -> Result<bool> {
        self.call_fn("consensus", ())
    }

    fn application_status(&self) -> Result<ApplicationStatus> {
        let jobs = self.babel_engine.get_jobs()?;
        let check_job_status = |name: &str, expected_status: JobStatus| {
            jobs.get(name)
                .map(|info| info.status == expected_status)
                .unwrap_or(false)
        };
        if check_job_status(UPLOAD_JOB_NAME, JobStatus::Running) {
            Ok(ApplicationStatus::Uploading)
        } else if check_job_status(DOWNLOAD_JOB_NAME, JobStatus::Running) {
            Ok(ApplicationStatus::Downloading)
        } else if find_const_in_ast::<PluginConfig>(&self.ast, PLUGIN_CONFIG_CONST_NAME).is_ok_and(
            |config| {
                config
                    .services
                    .iter()
                    .any(|service| check_job_status(&service.name, JobStatus::Pending))
            },
        ) {
            Ok(ApplicationStatus::Initializing)
        } else {
            Ok(from_dynamic(
                &self.call_fn::<_, Dynamic>("application_status", ())?,
            )?)
        }
    }

    fn sync_status(&self) -> Result<SyncStatus> {
        Ok(from_dynamic(
            &self.call_fn::<_, Dynamic>("sync_status", ())?,
        )?)
    }

    fn staking_status(&self) -> Result<StakingStatus> {
        Ok(from_dynamic(
            &self.call_fn::<_, Dynamic>("staking_status", ())?,
        )?)
    }

    fn call_custom_method(&self, name: &str, param: &str) -> Result<String> {
        self.call_fn(name, (param.to_string(),))
    }
}

fn into_rhai_result<T>(result: Result<T>) -> std::result::Result<T, Box<rhai::EvalAltResult>> {
    Ok(result.map_err(|err| <String as Into<rhai::EvalAltResult>>::into(format!("{err:#}")))?)
}

/// Helper structure that represents `JrpcRequest` from Rhai script perspective.
/// It allows any `Dynamic` object to be set as params. Then `rhai_plugin` takes care of json
/// serialization of `Map` or `Array`, since `babel_engine` expect params to be already
/// serialized to json string.
#[derive(Deserialize)]
pub struct BareJrpcRequest {
    pub host: String,
    pub method: String,
    pub params: Option<Dynamic>,
    pub headers: Option<HashMap<String, String>>,
}

impl TryInto<JrpcRequest> for BareJrpcRequest {
    type Error = Error;

    fn try_into(self) -> std::result::Result<JrpcRequest, Self::Error> {
        let params = match self.params {
            Some(value) => {
                if value.is_map() || value.is_array() {
                    Some(serde_json::to_string(&value)?)
                } else {
                    bail!("unsupported jrpc params type")
                }
            }
            None => None,
        };
        Ok(JrpcRequest {
            host: self.host,
            method: self.method,
            params,
            headers: self.headers,
        })
    }
}

/// Http response.
#[derive(Serialize, Clone)]
pub struct DressedHttpResponse {
    /// Http status code.
    pub status_code: u16,
    /// Response body as text.
    pub body: String,
}

impl DressedHttpResponse {
    pub fn expect_with(
        &mut self,
        check: FnPtr,
    ) -> std::result::Result<Map, Box<rhai::EvalAltResult>> {
        let engine = rhai::Engine::new();
        into_rhai_result(
            if check.call(&engine, &AST::empty(), (self.status_code,))? {
                Ok(engine.parse_json(&self.body, true)?)
            } else {
                Err(anyhow!("unexpected status_code: {}", self.status_code))
            },
        )
    }
    pub fn expect(&mut self, expected: u16) -> std::result::Result<Map, Box<rhai::EvalAltResult>> {
        into_rhai_result(if self.status_code == expected {
            Ok(rhai::Engine::new().parse_json(&self.body, true)?)
        } else {
            Err(anyhow!("unexpected status_code: {}", self.status_code))
        })
    }
}

impl From<HttpResponse> for DressedHttpResponse {
    fn from(value: HttpResponse) -> Self {
        Self {
            status_code: value.status_code,
            body: value.body,
        }
    }
}

/// Sh script response.
#[derive(Serialize, Clone)]
pub struct DressedShResponse {
    /// script exit code
    pub exit_code: i32,
    /// stdout
    pub stdout: String,
    /// stderr
    pub stderr: String,
}

impl DressedShResponse {
    pub fn unwrap(&mut self) -> std::result::Result<String, Box<rhai::EvalAltResult>> {
        into_rhai_result(if self.exit_code == 0 {
            Ok(self.stdout.clone())
        } else {
            Err(anyhow!("unexpected exit_code: {}", self.exit_code))
        })
    }
}

impl From<ShResponse> for DressedShResponse {
    fn from(value: ShResponse) -> Self {
        Self {
            exit_code: value.exit_code,
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        HttpResponse, JobConfig, JobInfo, JobStatus, JobType, JrpcRequest, RestRequest,
        RestartConfig, RestartPolicy, ShResponse,
    };
    use crate::metadata::{
        firewall, BabelConfig, NetConfiguration, NetType, RamdiskConfiguration, Requirements,
    };
    use eyre::bail;
    use mockall::*;

    mock! {
        pub BabelEngine {}

        impl Engine for BabelEngine {
            fn create_job(&self, job_name: &str, job_config: JobConfig) -> Result<()>;
            fn start_job(&self, job_name: &str) -> Result<()>;
            fn stop_job(&self, job_name: &str) -> Result<()>;
            fn job_info(&self, job_name: &str) -> Result<engine::JobInfo>;
            fn get_jobs(&self) -> Result<engine::JobsInfo>;
            fn run_jrpc(&self, req: JrpcRequest, timeout: Option<Duration>) -> Result<HttpResponse>;
            fn run_rest(&self, req: RestRequest, timeout: Option<Duration>) -> Result<HttpResponse>;
            fn run_sh(&self, body: &str, timeout: Option<Duration>) -> Result<ShResponse>;
            fn sanitize_sh_param(&self, param: &str) -> Result<String>;
            fn render_template(
                &self,
                template: &Path,
                output: &Path,
                params: &str,
            ) -> Result<()>;
            fn node_params(&self) -> HashMap<String, String>;
            fn save_data(&self, value: &str) -> Result<()>;
            fn load_data(&self) -> Result<String>;
            fn log(&self, level: Level, message: &str);
        }
    }

    #[test]
    fn test_capabilities() -> Result<()> {
        let script = r#"
        fn function_A(any_param) {
        }
        
        fn function_b(more, params) {
        }
        
        fn functionC() {
            // no params
        }
"#;
        let plugin = RhaiPlugin::new(script, MockBabelEngine::new())?;
        let mut expected_capabilities = vec![
            "function_A".to_string(),
            "function_b".to_string(),
            "functionC".to_string(),
        ];
        expected_capabilities.sort();
        let mut capabilities = plugin.capabilities();
        capabilities.sort();
        assert_eq!(expected_capabilities, capabilities);
        Ok(())
    }

    #[test]
    fn test_call_build_in_functions() -> Result<()> {
        let script = r#"
        fn init() {
            save_data("init");
        }

        fn upload() {
            save_data("upload");
        }

        fn height() {
            77
        }
        
        fn block_age() {
            18
        }
        
        fn name() {
            "block name"
        }

        fn address() {
            "node address"
        }

        fn consensus() {
            true
        }

        fn application_status() {
            "broadcasting"
        }

        fn sync_status() {
            "synced"
        }
        
        fn staking_status() {
            "staking"
        }
"#;
        let mut babel = MockBabelEngine::new();
        babel
            .expect_save_data()
            .with(predicate::eq("init"))
            .return_once(|_| Ok(()));
        babel
            .expect_save_data()
            .with(predicate::eq("upload"))
            .return_once(|_| Ok(()));
        babel
            .expect_get_jobs()
            .return_once(|| Ok(HashMap::default()));
        let plugin = RhaiPlugin::new(script, babel)?.clone(); // call clone() to make sure it works as well
        plugin.init()?;
        plugin.upload()?;
        assert_eq!(77, plugin.height()?);
        assert_eq!(18, plugin.block_age()?);
        assert_eq!("block name", &plugin.name()?);
        assert_eq!("node address", &plugin.address()?);
        assert!(plugin.consensus()?);
        assert_eq!(
            ApplicationStatus::Broadcasting,
            plugin.application_status()?
        );
        assert_eq!(SyncStatus::Synced, plugin.sync_status()?);
        assert_eq!(StakingStatus::Staking, plugin.staking_status()?);
        Ok(())
    }

    #[test]
    fn test_call_custom_method_call_engine() -> Result<()> {
        let script = r#"
    fn custom_method(param) {
        debug("debug message");
        info("info message");
        warn("warn message");
        error("error message");
        let out = parse_json(param).a;
        start_job("test_job_name", #{
            job_type: #{
                run_sh: "job body",
            },
            restart: #{
                "on_failure": #{
                    max_retries: 3,
                    backoff_timeout_ms: 1000,
                    backoff_base_ms: 500,
                },
            },
            needs: ["needed"],
        });
        create_job("download_job_name", #{
             job_type: #{
                download: #{},
            },
            restart: "never",
        });
        start_job("download_job_name");
        stop_job("test_job_name");
        out += "|" + job_info("test_job_name");
        out += "|" + get_jobs();
        out += "|" + run_jrpc(#{host: "host", method: "method", headers: #{"custom_header": "header value"}}).body;
        out += "|" + run_jrpc(#{host: "host", method: "method", params: #{"chain": "x"}}, 1).body;
        out += "|" + run_jrpc(#{host: "host", method: "method", params: ["positional", "args", "array"]}, 1).body;
        let http_out = run_rest(#{url: "url"});
        out += "|" + http_out.body;
        out += "|" + http_out.status_code;
        out += "|" + run_rest(#{url: "url", headers: #{"another-header": "another value"}}, 2).body;
        out += "|" + run_sh("body").stdout;
        let sh_out = run_sh("body", 3);
        out += "|" + sh_out.stderr;
        out += "|" + sh_out.exit_code;
        out += "|" + sanitize_sh_param("sh param");
        render_template("/template/path", "output/path.cfg", #{ PARAM1: "Value I"});
        out += "|" + node_params().to_json(); 
        save_data("some plugin data"); 
        out += "|" + load_data(); 
        out
    }
"#;
        let mut babel = MockBabelEngine::new();
        babel
            .expect_log()
            .with(
                predicate::eq(Level::DEBUG),
                predicate::eq("\"debug message\""),
            )
            .return_once(|_, _| ());
        babel
            .expect_log()
            .with(predicate::eq(Level::INFO), predicate::eq("info message"))
            .return_once(|_, _| ());
        babel
            .expect_log()
            .with(predicate::eq(Level::WARN), predicate::eq("warn message"))
            .return_once(|_, _| ());
        babel
            .expect_log()
            .with(predicate::eq(Level::ERROR), predicate::eq("error message"))
            .return_once(|_, _| ());
        babel
            .expect_create_job()
            .with(
                predicate::eq("test_job_name"),
                predicate::eq(JobConfig {
                    job_type: JobType::RunSh("job body".to_string()),
                    restart: RestartPolicy::OnFailure(RestartConfig {
                        backoff_timeout_ms: 1000,
                        backoff_base_ms: 500,
                        max_retries: Some(3),
                    }),
                    shutdown_timeout_secs: None,
                    shutdown_signal: None,
                    needs: Some(vec!["needed".to_string()]),
                }),
            )
            .return_once(|_, _| Ok(()));
        babel
            .expect_start_job()
            .with(predicate::eq("test_job_name"))
            .return_once(|_| Ok(()));
        babel
            .expect_create_job()
            .with(
                predicate::eq("download_job_name"),
                predicate::eq(JobConfig {
                    job_type: JobType::Download {
                        manifest: None,
                        destination: None,
                        max_connections: None,
                        max_runners: None,
                    },
                    restart: RestartPolicy::Never,
                    shutdown_timeout_secs: None,
                    shutdown_signal: None,
                    needs: None,
                }),
            )
            .return_once(|_, _| Ok(()));
        babel
            .expect_start_job()
            .with(predicate::eq("download_job_name"))
            .return_once(|_| Ok(()));
        babel
            .expect_stop_job()
            .with(predicate::eq("test_job_name"))
            .return_once(|_| Ok(()));
        babel
            .expect_job_info()
            .with(predicate::eq("test_job_name"))
            .return_once(|_| {
                Ok(JobInfo {
                    status: JobStatus::Finished {
                        exit_code: Some(1),
                        message: "error msg".to_string(),
                    },
                    progress: None,
                    restart_count: 0,
                    logs: vec![],
                    upgrade_blocking: false,
                })
            });
        babel.expect_get_jobs().return_once(|| {
            Ok(HashMap::from_iter([(
                "custom_name".to_string(),
                JobInfo {
                    status: JobStatus::Running,
                    progress: Default::default(),
                    restart_count: 0,
                    logs: vec![],
                    upgrade_blocking: true,
                },
            )]))
        });
        babel
            .expect_run_jrpc()
            .with(
                predicate::eq(JrpcRequest {
                    host: "host".to_string(),
                    method: "method".to_string(),
                    params: None,
                    headers: Some(HashMap::from_iter([(
                        "custom_header".to_string(),
                        "header value".to_string(),
                    )])),
                }),
                predicate::eq(None),
            )
            .return_once(|_, _| {
                Ok(HttpResponse {
                    status_code: 200,
                    body: "jrpc_response".to_string(),
                })
            });
        babel
            .expect_run_jrpc()
            .with(
                predicate::eq(JrpcRequest {
                    host: "host".to_string(),
                    method: "method".to_string(),
                    params: Some(r#"{"chain":"x"}"#.to_string()),
                    headers: None,
                }),
                predicate::eq(Some(Duration::from_secs(1))),
            )
            .return_once(|_, _| {
                Ok(HttpResponse {
                    status_code: 200,
                    body: "jrpc_with_map_and_timeout_response".to_string(),
                })
            });
        babel
            .expect_run_jrpc()
            .with(
                predicate::eq(JrpcRequest {
                    host: "host".to_string(),
                    method: "method".to_string(),
                    params: Some(r#"["positional","args","array"]"#.to_string()),
                    headers: None,
                }),
                predicate::eq(Some(Duration::from_secs(1))),
            )
            .return_once(|_, _| {
                Ok(HttpResponse {
                    status_code: 200,
                    body: "jrpc_with_array_and_timeout_response".to_string(),
                })
            });
        babel
            .expect_run_rest()
            .with(
                predicate::eq(RestRequest {
                    url: "url".to_string(),
                    headers: None,
                }),
                predicate::eq(None),
            )
            .return_once(|_, _| {
                Ok(HttpResponse {
                    status_code: 200,
                    body: "rest_response".to_string(),
                })
            });
        babel
            .expect_run_rest()
            .with(
                predicate::eq(RestRequest {
                    url: "url".to_string(),
                    headers: Some(HashMap::from_iter([(
                        "another-header".to_string(),
                        "another value".to_string(),
                    )])),
                }),
                predicate::eq(Some(Duration::from_secs(2))),
            )
            .return_once(|_, _| {
                Ok(HttpResponse {
                    status_code: 200,
                    body: "rest_with_timeout_response".to_string(),
                })
            });
        babel
            .expect_run_sh()
            .with(predicate::eq("body"), predicate::eq(None))
            .return_once(|_, _| {
                Ok(ShResponse {
                    exit_code: 0,
                    stdout: "sh_response".to_string(),
                    stderr: "".to_string(),
                })
            });
        babel
            .expect_run_sh()
            .with(
                predicate::eq("body"),
                predicate::eq(Some(Duration::from_secs(3))),
            )
            .return_once(|_, _| {
                Ok(ShResponse {
                    exit_code: -1,
                    stdout: "".to_string(),
                    stderr: "sh_with_timeout_err".to_string(),
                })
            });
        babel
            .expect_sanitize_sh_param()
            .with(predicate::eq("sh param"))
            .return_once(|_| Ok("sh_sanitized".to_string()));
        babel
            .expect_render_template()
            .with(
                predicate::eq(Path::new("/template/path")),
                predicate::eq(Path::new("output/path.cfg")),
                predicate::eq(r#"{"PARAM1":"Value I"}"#),
            )
            .return_once(|_, _, _| Ok(()));
        babel
            .expect_node_params()
            .return_once(|| HashMap::from_iter([("key_A".to_string(), "value_A".to_string())]));
        babel
            .expect_save_data()
            .with(predicate::eq("some plugin data"))
            .return_once(|_| Ok(()));
        babel
            .expect_load_data()
            .return_once(|| Ok("loaded data".to_string()));

        let plugin = RhaiPlugin::new(script, babel)?;
        assert_eq!(
            r#"json_as_param|#{"logs": [], "progress": (), "restart_count": 0, "status": #{"finished": #{"exit_code": 1, "message": "error msg"}}, "upgrade_blocking": false}|#{"custom_name": #{"logs": [], "progress": (), "restart_count": 0, "status": "running", "upgrade_blocking": true}}|jrpc_response|jrpc_with_map_and_timeout_response|jrpc_with_array_and_timeout_response|rest_response|200|rest_with_timeout_response|sh_response|sh_with_timeout_err|-1|sh_sanitized|{"key_A":"value_A"}|loaded data"#,
            plugin.call_custom_method("custom_method", r#"{"a":"json_as_param"}"#)?
        );
        Ok(())
    }

    #[test]
    fn test_errors_handling() -> Result<()> {
        let script = r#"
        const METADATA = #{
    // invalid metadata
};

    fn custom_method_with_exception(param) {
        throw "some Rhai exception";
        ""
    }
    
    fn custom_failing_method(param) {
        load_data()
    }
"#;
        let mut babel = MockBabelEngine::new();
        babel
            .expect_load_data()
            .return_once(|| bail!("some Rust error"));
        let plugin = RhaiPlugin::new(script, babel)?;
        assert_eq!(
            "Rhai function 'custom_method_with_exception' returned error",
            plugin
                .call_custom_method("custom_method_with_exception", "")
                .unwrap_err()
                .to_string()
        );
        assert!(format!(
            "{:#}",
            plugin
                .call_custom_method("custom_failing_method", "")
                .unwrap_err()
        )
        .starts_with(
            "Rhai function 'custom_failing_method' returned error: Runtime error: some Rust error"
        ));
        assert_eq!(
            "Rhai function 'no_method' returned error: Function not found: no_method",
            format!(
                "{:#}",
                plugin.call_custom_method("no_method", "").unwrap_err()
            )
        );
        assert_eq!(
            "Invalid Rhai script - failed to deserialize METADATA",
            plugin.metadata().unwrap_err().to_string()
        );
        Ok(())
    }

    #[test]
    fn test_metadata() -> Result<()> {
        let meta_rhai = r#"
const METADATA = #{
    // comments are allowed
    kernel: "5.10.174-build.1+fc.ufw",
    min_babel_version: "0.0.9",
    node_version: "node_v",
    protocol: "proto",
    node_type: "n_type",
    description: "node description",
    requirements: #{
        vcpu_count: 2,
        mem_size_mb: 1024,
        disk_size_gb: 10,
    },
    nets: #{
        test: #{
            url: "test_url",
            net_type: "test",
            some_custom: "metadata",
        },
        main: #{
            url: "main_url",
            net_type: "main",
        },
    },
    babel_config: #{
        log_buffer_capacity_ln: 1024,
        swap_size_mb: 1024,
        ramdisks: [
            #{
                ram_disk_mount_point: "/mnt/ramdisk",
                ram_disk_size_mb: 512,
            },
        ],
    },
    firewall: #{
        enabled: true,
        default_in: "deny",
        default_out: "allow",
        rules: [
            #{
                name: "Rule A",
                action: "allow",
                direction: "in",
                protocol: "tcp",
                ips: "192.168.0.1/24",
                ports: [77, 1444, 8080],
            },
            #{
                name: "Rule B",
                action: "deny",
                direction: "out",
                protocol: "udp",
                ips: "192.167.0.1/24",
                ports: [77],
            },
            #{
                name: "Rule C",
                action: "reject",
                direction: "out",
                ips: "192.169.0.1/24",
                ports: [],
            },
        ],
    },
};
fn any_function() {}
"#;
        let meta_rust = BlockchainMetadata {
            kernel: "5.10.174-build.1+fc.ufw".to_string(),
            node_version: "node_v".to_string(),
            protocol: "proto".to_string(),
            node_type: "n_type".to_string(),
            description: Some("node description".to_string()),
            requirements: Requirements {
                vcpu_count: 2,
                mem_size_mb: 1024,
                disk_size_gb: 10,
            },
            nets: HashMap::from_iter([
                (
                    "test".to_string(),
                    NetConfiguration {
                        url: "test_url".to_string(),
                        net_type: NetType::Test,
                        meta: HashMap::from_iter([(
                            "some_custom".to_string(),
                            "metadata".to_string(),
                        )]),
                    },
                ),
                (
                    "main".to_string(),
                    NetConfiguration {
                        url: "main_url".to_string(),
                        net_type: NetType::Main,
                        meta: Default::default(),
                    },
                ),
            ]),
            min_babel_version: "0.0.9".to_string(),
            babel_config: BabelConfig {
                log_buffer_capacity_ln: 1024,
                swap_size_mb: 1024,
                swap_file_location: "/swapfile".to_string(),
                ramdisks: Some(vec![RamdiskConfiguration {
                    ram_disk_mount_point: "/mnt/ramdisk".to_string(),
                    ram_disk_size_mb: 512,
                }]),
            },
            firewall: firewall::Config {
                enabled: true,
                default_in: firewall::Action::Deny,
                default_out: firewall::Action::Allow,
                rules: vec![
                    firewall::Rule {
                        name: "Rule A".to_string(),
                        action: firewall::Action::Allow,
                        direction: firewall::Direction::In,
                        protocol: Some(firewall::Protocol::Tcp),
                        ips: Some("192.168.0.1/24".to_string()),
                        ports: vec![77, 1444, 8080],
                    },
                    firewall::Rule {
                        name: "Rule B".to_string(),
                        action: firewall::Action::Deny,
                        direction: firewall::Direction::Out,
                        protocol: Some(firewall::Protocol::Udp),
                        ips: Some("192.167.0.1/24".to_string()),
                        ports: vec![77],
                    },
                    firewall::Rule {
                        name: "Rule C".to_string(),
                        action: firewall::Action::Reject,
                        direction: firewall::Direction::Out,
                        protocol: None,
                        ips: Some("192.169.0.1/24".to_string()),
                        ports: vec![],
                    },
                ],
            },
        };
        let plugin = RhaiPlugin::new(meta_rhai, MockBabelEngine::new())?;
        assert_eq!(meta_rust, plugin.metadata()?);
        Ok(())
    }
}
