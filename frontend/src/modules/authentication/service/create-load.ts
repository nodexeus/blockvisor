import { config } from '../consts';
import type { Load } from '@sveltejs/kit';
import type { TAuthFlow, FlowTypeId } from '../models';

export const createLoad =
  (flowType: FlowTypeId): Load =>
  async ({ url, fetch, session }) => {
    if (session.user) {
      return {
        status: 302,
        redirect: `/verify`,
      };
    }

    const flowID = url.searchParams.get('flow');
    const res = await fetch(`/api/auth/${flowType}`, {
      headers: { flow_id: flowID },
      credentials: 'include',
    });

    if (!res.ok) {
      return {
        status: 302,
        redirect: `${config.auth.publicUrl}/self-service/${flowType}/browser`,
      };
    }

    const { data: flow }: { status: number; data: TAuthFlow } =
      await res.json();

    return {
      props: { authUi: flow.ui },
    };
  };
