import { REFRESH_TOKEN } from '../const';

type Callback = (data: CreateUserData) => void;

export const refreshUserToken = (refresh: string, callback: Callback) => {
  fetch(REFRESH_TOKEN, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ refresh }),
  })
    .then((res) => res.json())
    .then(callback)
    .catch((error) => console.error(error));
};
