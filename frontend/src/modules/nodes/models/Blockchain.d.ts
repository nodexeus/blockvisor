interface Blockchain {
  id?: string;
  name: string;
  description?: string;
  status: 'development' | 'alpha' | 'beta' | 'production' | 'deleted';
  project_url?: string;
  token?: string;
  repo_url?: string;
  supports_etl: boolean;
  supports_node: boolean;
  supports_staking: boolean;
  supports_broadcast: boolean;
  version?: string;
  created_at?: string;
  updated_at?: string;
}
