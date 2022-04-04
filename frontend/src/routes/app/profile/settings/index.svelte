<script lang="ts" context="module">
  import type { Load } from '@sveltejs/kit';

  export const load: Load = async ({ session, fetch }) => {
    if (!session || !session.user) return { status: 302, redirect: '/' };
    const settingsFlowResult = await fetch(`/api/auth/initiate-settings`, {
      credentials: 'include',
    });
    if (settingsFlowResult.ok) {
      const { data } = await settingsFlowResult.json();
      return {
        props: { authUi: data.ui },
      };
    }
    return {};
  };
</script>

<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';
  import PasswordForm from 'modules/settings/components/PasswordForm/PasswordForm.svelte';
  import InfoForm from 'modules/settings/components/InfoForm/InfoForm.svelte';
  import FormSection from 'modules/settings/components/FormSection/FormSection.svelte';
  import AuthNotes from 'modules/authentication/components/AuthNotes/AuthNotes.svelte';
  import AuthErrors from 'modules/authentication/components/AuthErrors/AuthErrors.svelte';
  export let authUi: UiContainer;

  $: ui = authUi;
</script>

<section class="settings__header grid grid-spacing--small-only">
  <div class="settings__header-wrapper">
    <h1 class="t-xxlarge-fluid s-bottom--large settings__title">Settings</h1>
    <p class="s-bottom-xlarge t-color-text-4 t-tiny">
      You successfully recovered your account. Please change your password
      within the next 15.00 minutes.
    </p>
  </div>
</section>
<section class="settings__content grid grid-spacing--small-only">
  <FormSection title="Personal Information">
    <InfoForm {ui} />
    <AuthNotes {ui} />
    <AuthErrors {ui} />
  </FormSection>
  <FormSection title="Password">
    <PasswordForm {ui} />
  </FormSection>
</section>

<style>
  .settings {
    &__title {
      margin-top: 0;
    }

    &__header {
      margin-bottom: 60px;
      &-wrapper {
        grid-column: 2/10;

        @media (--screen-large) {
          grid-column: 2/6;
        }
      }
    }
  }
</style>
