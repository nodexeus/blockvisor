<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Logo from 'components/Logo/Logo.svelte';
  import { ROUTES } from 'consts/routes';
  import QuoteBlock from 'modules/app/components/QuoteBlock/QuoteBlock.svelte';
</script>

<main class="not-found">
  <QuoteBlock>
    <svelte:fragment slot="content">
      <figure class="s-bottom--large">
        <Logo type="blockjoy-large" />
      </figure>
      <h2 class="t-uppercase t-huge-fluid s-bottom--xxlarge">
        <strong>404</strong><br />
        <strong> Page Not Found </strong>
      </h2>
      <p class="t-large-fluid s-bottom--large">
        This page might be missing or is no longer active.
      </p>
      <Button asLink href={ROUTES.LOGIN} style="primary">Return home</Button>
    </svelte:fragment>
  </QuoteBlock>
</main>

<style>
  .not-found {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: center;

    & > :global(*) {
      display: flex;
      flex-direction: column;
      justify-content: center;
      min-height: 100%;
      flex-grow: 1;
    }
  }
</style>
