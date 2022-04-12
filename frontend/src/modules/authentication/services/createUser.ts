import { CREATE_USER } from '../const';

type Callback = (data: CreateUserData) => void;

export const createUser = (data: CreateUserData, callback: Callback) => {
  fetch(CREATE_USER, {
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
