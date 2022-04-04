<script lang="ts" context="module">
  import { createLoad } from 'modules/authentication/service';
  export const load = createLoad('login');
</script>

<script lang="ts">
  import LoginForm from 'modules/authentication/components/LoginForm/LoginForm.svelte';
  import Layout from 'modules/authentication/components/Layout/Layout.svelte';
  import { MetaTags } from 'svelte-meta-tags';
  import type { UiContainer } from '@ory/kratos-client';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import AuthErrors from 'modules/authentication/components/AuthErrors/AuthErrors.svelte';
  import AuthNotes from 'modules/authentication/components/AuthNotes/AuthNotes.svelte';
  import { ROUTES } from 'consts/routes';

  export let authUi: UiContainer;
  $: ui = authUi;
</script>

<MetaTags title="Login | BlockVisor" />

<PublicRoute>
  <Layout title="Login">
    <LoginForm {ui} />
    <AuthNotes {ui} />
    <AuthErrors {ui} />
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
