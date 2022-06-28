<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import { ROUTES } from 'consts/routes';
  import SetNewPasswordForm from 'modules/authentication/components/SetNewPasswordForm/SetNewPasswordForm.svelte';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import { MetaTags } from 'svelte-meta-tags';
</script>

<MetaTags title="Forgot your password? | BlockVisor" />

<PublicRoute>
  <p
    class="t-small t-color-text-3 s-bottom--medium forgot-password__description"
  >
    Set up your new password and makes sure you store it in a safe place.
  </p>
  <div class="s-bottom--medium-small">
    <SetNewPasswordForm />
  </div>

  <Button href={ROUTES.LOGIN} display="block" size="small" style="ghost" asLink
    >Cancel</Button
  >
</PublicRoute>
