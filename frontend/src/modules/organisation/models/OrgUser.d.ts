export interface OrgUser {
  user_id: string;
  org_id: string;
  is_personal: boolean;
  role: 'owner' | 'user';
  created_at: boolean;
  updated_at: boolean;
}
