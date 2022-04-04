<script lang="ts" context="module">
  import { createLoad } from 'modules/authentication/service';
  export const load = createLoad('recovery');
</script>

<script lang="ts">
  import ForgotPasswordForm from 'modules/authentication/components/ForgotPasswordForm/ForgotPasswordForm.svelte';
  import { MetaTags } from 'svelte-meta-tags';
  import type { UiContainer } from '@ory/kratos-client';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import AuthErrors from 'modules/authentication/components/AuthErrors/AuthErrors.svelte';
  import AuthNotes from 'modules/authentication/components/AuthNotes/AuthNotes.svelte';
  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';
  export let authUi: UiContainer;

  $: ui = authUi;
</script>

<MetaTags title="Forgot your password? | BlockVisor" />

<PublicRoute>
  <p
    class="t-small t-color-text-3 s-bottom--medium forgot-password__description"
  >
    No worries, just enter your registration e-mail address and we will send you
    a link to reset your password.
  </p>
  <ForgotPasswordForm {ui} />
  <Button href={ROUTES.LOGIN} display="block" size="small" style="ghost" asLink
    >Cancel</Button
  >
  <AuthNotes {ui} />
  <AuthErrors {ui} />
</PublicRoute>
