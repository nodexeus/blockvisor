<script lang="ts">
  import BlockJoyLogoSmall from 'logos/blockjoy-logo-small.svg';
  import BlockJoyLogo from 'logos/blockjoy-logo-large.svg';

  import { isUserVerified } from 'modules/authorization/utils';
  import { user } from 'modules/authentication/store';
  import { ROUTES } from 'consts/routes';
  import { FEATURE_FLAGS } from 'consts/featureFlags';

  export let type: 'blockvisor-small' | 'faded' | 'blockjoy-large' =
    'blockvisor-small';
  export let center = false;

  $: href = isUserVerified($user)
    ? FEATURE_FLAGS.BLOCKVISOR
      ? ROUTES.DASHBOARD
      : ROUTES.BROADCASTS
    : ROUTES.LOGIN;
</script>

<h1 class="logo" class:logo--center={center}>
  <a {href} class="logo__link">
    {#if type === 'blockjoy-large'}
      <span class="visually-hidden">BlockJoy</span>
      <BlockJoyLogo />
    {:else if type === 'faded'}
      <span class="logo__faded">
        <BlockJoyLogoSmall />
      </span>
    {:else}
      <span class="logo__primary">
        <span class="visually-hidden">BlockJoy</span>
        <BlockJoyLogoSmall />
      </span>
    {/if}
  </a>
</h1>

<style>
  .logo {
    margin: 0;

    &__link {
      display: inline-block;
      transition: opacity 0.18s var(--transition-easing-cubic);

      &:hover,
      &:active {
        opacity: 0.75;
      }
    }

    &--center {
      display: flex;
      justify-content: center;
    }
  }

  .logo__primary {
    fill: var(--color-primary);
  }

  .logo__faded {
    fill: #f8faf6;
    opacity: 0.3;
  }
</style>
