interface Broadcast {
  id?: string;
  blockchain_id: string;
  org_id: string;
  name: string;
  addresses: string;
  callback_url: string;
  auth_token: string;
  txn_types: string;
  is_active: boolean;
  last_processes_height: number;
  created_at: string;
  updated_at: string;
}
