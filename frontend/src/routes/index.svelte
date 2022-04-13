<script lang="ts">
  import Layout from 'modules/authentication/components/Layout/Layout.svelte';
  import { MetaTags } from 'svelte-meta-tags';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import { ROUTES } from 'consts/routes';
</script>

<MetaTags title="Login | BlockVisor" />

<PublicRoute>
  <Layout title="Login">
    <footer class="login-footer t-tiny">
      <div class="t-right">
        <a class="link" href={ROUTES.FORGOT_PASSWORD}>Forgot password?</a>
      </div>
      <div class="login-footer__account">
        <p class="t-color-text-2">Don't have a BlockVisor account?</p>
        <a href={ROUTES.REGISTER} class="link link--primary"
          >Create an Account</a
        >
      </div>
    </footer>
  </Layout>
</PublicRoute>

<style>
  .login-footer {
    margin-top: 12px;

    &__account {
      margin-top: 40px;
      padding-top: 20px;
      border-top: 1px solid theme(--color-text-5-o10);
    }
  }
</style>
