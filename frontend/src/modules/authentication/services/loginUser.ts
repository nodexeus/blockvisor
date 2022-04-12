import { LOGIN_USER } from '../const';

type Callback = (data: UserSession) => void;

export const loginUser = (data: UserData, callback: Callback) => {
  fetch(LOGIN_USER, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data),
  })
    .then((res) => res.json())
    .then(callback)
    .catch((error) => console.error(error));
};
