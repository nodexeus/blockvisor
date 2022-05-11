import axios from 'axios';
import { REFRESH_TOKEN } from 'modules/authentication/const';
import { getRefreshToken } from './ServerRequest';
const httpClient = axios.create();

httpClient.interceptors.response.use(
  async (response) => response,
  async (error) => {
    const originalRequest = error.config;
    if (error.response.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      const refreshToken = getRefreshToken(originalRequest);

      try {
        const res = await httpClient.post(REFRESH_TOKEN, {
          refresh: refreshToken,
        });
        const data = res.data;

        originalRequest.headers['Authorization'] = 'Bearer ' + data.token;

        return httpClient(originalRequest);
      } catch (error) {
        return Promise.reject(error);
      }
    }
    return Promise.reject(error);
  },
);
export { httpClient };
