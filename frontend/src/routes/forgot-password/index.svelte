<script lang="ts">
  import { MetaTags } from 'svelte-meta-tags';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';
  import ForgotPasswordForm from 'modules/authentication/components/ForgotPasswordForm/ForgotPasswordForm.svelte';
</script>

<MetaTags title="Forgot your password? | BlockVisor" />

<PublicRoute>
  <p
    class="t-small t-color-text-3 s-bottom--medium forgot-password__description"
  >
    No worries, just enter your registration e-mail address and we will send you
    a link to reset your password.
  </p>
  <div class="s-bottom--medium-small">
    <ForgotPasswordForm />
  </div>

  <Button href={ROUTES.LOGIN} display="block" size="small" style="ghost" asLink
    >Cancel</Button
  >
</PublicRoute>
