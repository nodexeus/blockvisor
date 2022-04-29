<script lang="ts">
  import { MetaTags } from 'svelte-meta-tags';
  import PublicRoute from 'modules/authorization/components/PublicRoute/PublicRoute.svelte';
  import { ROUTES } from 'consts/routes';
  import RegisterForm from 'modules/authentication/components/RegisterForm/RegisterForm.svelte';
</script>

<MetaTags title="Register | BlockVisor" />

<PublicRoute>
  <RegisterForm />
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
