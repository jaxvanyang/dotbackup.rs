#[derive(Debug, Clone, Default)]
pub enum Action {
	#[default]
	Backup,
	Setup,
	Help,
	List,
	Version,
	DumpConfig,
}
