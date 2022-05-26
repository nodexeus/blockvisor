import { browser } from '$app/env';
import { goto } from '$app/navigation';
import { FEATURE_FLAGS } from 'consts/featureFlags';
import { ROUTES } from 'consts/routes';
import type { UserInfo } from 'utils';

export const checkAndRedirect = (
  user: UserSession | UserInfo,
  type: 'private' | 'public',
) => {
  const shouldRedirect =
    type === 'private' ? !user?.id || !user.verified : !!user?.id;

  /**
   * TODO handle !user.verified
   */

  if (!browser || !shouldRedirect) {
    return false;
  }

  let redirectTo =
    type === 'private'
      ? ROUTES.LOGIN
      : FEATURE_FLAGS.BLOCKVISOR
      ? ROUTES.DASHBOARD
      : ROUTES.BROADCASTS;

  if (type === 'private' && !user?.verified) {
    redirectTo = ROUTES.VERIFY;
  }

  goto(redirectTo, { replaceState: true });
};
