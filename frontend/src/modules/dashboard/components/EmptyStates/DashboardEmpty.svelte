<script lang="ts">
  import anime from 'animejs';
  import { fade } from 'svelte/transition';

  import Button from 'components/Button/Button.svelte';
  import { onMount } from 'svelte';
  import ActionRow from '../ActionRow/ActionRow.svelte';
  import AnimatedGraphic from '../AnimatedGraphic/AnimatedGraphic.svelte';
  import { ROUTES } from 'consts/routes';

  onMount(() => {
    anime({
      targets: '#js-title',
      easing: 'easeOutExpo',
      opacity: [0, 1],
      translateY: [12, 0],
      duration: 1000,
      delay: 100,
    });

    anime({
      targets: '#js-action-rows > li',
      easing: 'easeOutExpo',
      opacity: [0, 1],
      translateY: [12, 0],
      delay: (_, i) => 100 + 300 * (i + 1),
      duration: 800,
    });
  });
</script>

<section
  class="grid-spacing dashboard--empty"
  in:fade={{ delay: 250, duration: 100 }}
>
  <div id="js-title" class="dashboard--empty__title">
    <h2 class="t-xlarge-fluid">Get Started With Blockvisor!</h2>
    <AnimatedGraphic />
  </div>

  <ul id="js-action-rows" class="u-list-reset dashboard--empty__content">
    <li>
      <ActionRow>
        <svelte:fragment slot="title">Create a New Node</svelte:fragment>
        <svelte:fragment slot="description"
          >Add your nodes and hosts to get started with BlockVisor.</svelte:fragment
        >
        <Button
          style="secondary"
          size="small"
          slot="action"
          asLink
          href={ROUTES.NODE_ADD}
        >
          Add Node</Button
        >
      </ActionRow>
    </li>
    <li>
      <ActionRow>
        <svelte:fragment slot="title">Create a New Host</svelte:fragment>
        <svelte:fragment slot="description"
          >Add your nodes and hosts to get started with BlockVisor.</svelte:fragment
        >
        <Button
          style="secondary"
          size="small"
          slot="action"
          asLink
          href={ROUTES.HOST_ADD}
        >
          Add Host</Button
        >
      </ActionRow>
    </li>
  </ul>
</section>

<style>
  .dashboard--empty {
    min-height: 80%;
    max-width: 512px;
    margin: 40px auto;

    @media (--screen-medium-large) {
      margin: 0 auto;
      display: flex;
      flex-direction: column;
      justify-content: center;
    }

    &__content {
      :global(> li) {
        opacity: 0;
      }

      :global(> li + li) {
        border-top: 1px solid theme(--color-text-5-o10);
      }
    }

    &__title {
      display: flex;
      gap: 32px;
      margin-bottom: 40px;
      flex-direction: column;
      align-items: center;
      text-align: center;
      opacity: 0;

      @media (--screen-medium-small) {
        text-align: left;
        gap: 56px;
        justify-content: space-between;
        flex-direction: row;
      }

      @media (--screen-large) {
        grid-column: 4/9;

        & :global(figure) {
          flex: 0 0 160px;
        }
      }
    }
  }
</style>
