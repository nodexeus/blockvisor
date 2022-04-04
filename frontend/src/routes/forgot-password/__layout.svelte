<script lang="ts">
  import Layout from 'modules/authentication/components/Layout/Layout.svelte';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
</script>

<PublicRoute>
  <Layout title="Forgot Your Password?">
    <slot />
  </Layout>
</PublicRoute>
