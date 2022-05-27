import type { APPS } from 'models/App';

export interface App {
  breadcrumbs: Breadcrumb[];
  activeApp: APPS;
  user: UserInfo;
}
