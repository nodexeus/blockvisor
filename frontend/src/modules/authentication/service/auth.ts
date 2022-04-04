import { Configuration, V0alpha2Api } from '@ory/kratos-client';

import { config } from '../consts';

export const authFlowMap = {
  registration: 'getSelfServiceRegistrationFlow',
  recovery: 'getSelfServiceRecoveryFlow',
  verification: 'getSelfServiceVerificationFlow',
  settings: 'getSelfServiceSettingsFlow',
  error: 'getSelfServiceError',
  login: 'getSelfServiceLoginFlow',
};

export const authApi = new V0alpha2Api(
  new Configuration({
    basePath: config.auth.publicUrl as string,
    baseOptions: {
      withCredentials: true,
    },
  }),
);
export const authAdminApi = new V0alpha2Api(
  new Configuration({
    basePath: config.auth.adminUrl as string,
    baseOptions: {
      withCredentials: true,
    },
  }),
);
