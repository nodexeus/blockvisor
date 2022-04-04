<script lang="ts" context="module">
  import { createLoad } from 'modules/authentication/service';
  export const load = createLoad('registration');
</script>

<script lang="ts">
  import RegisterForm from 'modules/authentication/components/RegisterForm/RegisterForm.svelte';
  import { MetaTags } from 'svelte-meta-tags';
  import type { UiContainer } from '@ory/kratos-client';
  import AuthErrors from 'modules/authentication/components/AuthErrors/AuthErrors.svelte';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import AuthNotes from 'modules/authentication/components/AuthNotes/AuthNotes.svelte';
  import { ROUTES } from 'consts/routes';
  export let authUi: UiContainer;

  $: ui = authUi;
</script>

<MetaTags title="Register | BlockVisor" />

<PublicRoute>
  <RegisterForm {ui} />
  <AuthNotes {ui} />
  <AuthErrors {ui} />
  <footer class="register-footer t-tiny">
    <div class="register-footer__account">
      <p class="t-color-text-2">Already have a BlockVisor account?</p>
      <a href={ROUTES.LOGIN} class="link link--primary">Log In</a>
    </div>
  </footer>
</PublicRoute>

<style>
  .register-footer {
    margin-top: 12px;

    &__account {
      margin-top: 40px;
      padding-top: 20px;
      border-top: 1px solid theme(--color-text-5-o10);
    }
  }
</style>
