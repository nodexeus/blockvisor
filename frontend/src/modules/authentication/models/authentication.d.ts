interface UserData {
  email: string;
  password: string;
}

interface CreateUserData extends UserData {
  password_confirm: string;
}

interface UserSession {
  id: string;
  email: string;
  refresh: string;
  token: string;
  created_at: string;
}
