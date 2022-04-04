import type { Load } from '@sveltejs/kit';

export const load: Load = async ({ session, fetch }) => {
  if (!session.user) return {};

  const result = await fetch(`/api/auth/initiate-logout`, {
    credentials: 'include',
  });

  const { data } = await result.json();
  return {
    props: {
      logoutUrl: data.logout_url,
    },
  };
};
