<script lang="ts">
  import BaseLayout from 'modules/app/components/BaseLayout/BaseLayout.svelte';
  import MasterLayout from 'modules/app/components/MasterLayout/MasterLayout.svelte';
  import PrivateRoute from 'modules/authorization/components/PrivateRoute/PrivateRoute.svelte';
</script>

<BaseLayout>
  <PrivateRoute>
    <MasterLayout>
      <slot />
    </MasterLayout>
  </PrivateRoute>
</BaseLayout>
