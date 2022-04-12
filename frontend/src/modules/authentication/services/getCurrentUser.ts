import { GET_USER } from '../const';

type Callback = (data: CreateUserData) => void;

export const getCurrentUser = (callback: Callback) => {
  fetch(GET_USER, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
    },
  })
    .then((res) => res.json())
    .then(callback)
    .catch((error) => console.error(error));
};
