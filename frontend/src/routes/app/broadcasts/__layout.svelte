<script lang="ts">
  import LayoutSingleColumn from 'components/LayoutSingleColumn/LayoutSingleColumn.svelte';
  import { pageTransition } from 'consts/animations';

  import { ROUTES } from 'consts/routes';
  import { APPS } from 'models/App';

  import { app } from 'modules/app/store';
  import SubscriptionWrapper from 'modules/subscriptions/components/SubscriptionWrapper/SubscriptionWrapper.svelte';
  import { onMount } from 'svelte';

  onMount(() => {
    app.setActiveApp(APPS.BROADCAST);
    app.setBreadcrumbs([
      {
        title: 'Automation',
        url: ROUTES.BROADCASTS,
      },
      {
        title: 'All Broadcasts',
        url: '',
      },
    ]);
  });
</script>

<LayoutSingleColumn transition={pageTransition}>
  <SubscriptionWrapper hasSubscription={true}>
    <slot />
  </SubscriptionWrapper>
</LayoutSingleColumn>
