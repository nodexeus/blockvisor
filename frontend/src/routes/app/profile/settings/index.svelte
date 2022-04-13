<script lang="ts">
  import FormSection from 'modules/settings/components/FormSection/FormSection.svelte';
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
  <FormSection title="Personal Information" />
  <FormSection title="Password">Password form</FormSection>
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
