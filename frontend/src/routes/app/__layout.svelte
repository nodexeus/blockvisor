<script lang="ts">
  import BaseLayout from 'modules/app/components/BaseLayout/BaseLayout.svelte';
  import MasterLayout from 'modules/app/components/MasterLayout/MasterLayout.svelte';
  import PrivateRoute from 'modules/authorization/components/PrivateRoute/PrivateRoute.svelte';
  import { setContext } from 'svelte';
  export let logoutUrl = '';
  setContext('logoutUrl', logoutUrl);
</script>

<BaseLayout>
  <PrivateRoute>
    <MasterLayout>
      <slot />
    </MasterLayout>
  </PrivateRoute>
</BaseLayout>
