import type {
  SelfServiceLoginFlow,
  SelfServiceRecoveryFlow,
  SelfServiceRegistrationFlow,
  SelfServiceSettingsFlow,
  SelfServiceVerificationFlow,
} from '@ory/kratos-client';

export type TAuthFlow =
  | SelfServiceLoginFlow
  | SelfServiceRegistrationFlow
  | SelfServiceRecoveryFlow
  | SelfServiceSettingsFlow
  | SelfServiceVerificationFlow;

export type FlowTypeId =
  | 'registration'
  | 'login'
  | 'settings'
  | 'verification'
  | 'recovery'
  | 'error';
