<script lang="ts">
  import Layout from 'modules/authentication/components/Layout/Layout.svelte';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
</script>

<PublicRoute>
  <Layout title="Set a New Password">
    <slot />
  </Layout>
</PublicRoute>
