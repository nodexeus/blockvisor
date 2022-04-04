import { authApi } from 'modules/authentication/service/auth';
import type {
  SelfServiceLogoutUrl,
  SelfServiceSettingsFlow,
} from '@ory/kratos-client';
import type { Request } from '@sveltejs/kit';

interface GetResponse {
  body: {
    data: SelfServiceLogoutUrl | SelfServiceSettingsFlow | string;
  };
  status: number;
  headers: {
    [key: string]: string;
  };
}

export const createInitiator = (method: string, isSettingsFlow = true) => {
  return async ({ request: req }: Request): Promise<GetResponse> => {
    const cookies = req.headers.get('cookie');
    try {
      // Temporary workaround until the settings function signature is consistent
      const promise = isSettingsFlow
        ? authApi[method](undefined, {
            withCredentials: true,
            headers: { cookie: cookies },
          })
        : authApi[method](cookies);
      const response = await promise;
      return {
        body: {
          data: response.data,
        },
        headers: {
          'Content-Type': 'application/json',
        },
        status: response.status,
      };
    } catch (err) {
      return {
        status: err.response.data.error.code,
        headers: {
          'Content-Type': 'application/json',
        },
        body: {
          data: `Could not initiate flow: ${err.response.data.error.message}`,
        },
      };
    }
  };
};
