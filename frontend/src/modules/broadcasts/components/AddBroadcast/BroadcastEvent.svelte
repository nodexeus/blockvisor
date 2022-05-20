<script lang="ts">
  import Checkbox from 'modules/forms/components/Checkbox/Checkbox.svelte';
  import { useForm } from 'svelte-use-form';
  import IconEye from 'icons/eye-12.svg';

  export let name: string;
  export let value: string;

  let details: boolean = false;

  function toggleDetails() {
    details = !details;
  }

  const form = useForm();
</script>

<div>
  <div class="add-broadcast__checkbox">
    <Checkbox {name} field={$form?.addGateway}>
      {value}
    </Checkbox>
    <button
      type="button"
      on:click={toggleDetails}
      class={`u-button-reset add-broadcast__action ${details && 'active'}`}
      ><IconEye /></button
    >
  </div>
  {#if details}
    <pre class="code-block">
      Lorem ipsum dolor sit amet consectetur adipisicing elit. Quos explicabo
      tempore, sed sequi, laudantium vel voluptates et repudiandae, debitis odit
      modi ex quam culpa natus voluptatum enim amet nesciunt similique.
    </pre>
  {/if}
</div>

<style>
  .add-broadcast__checkbox {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .add-broadcast__action {
    color: var(--color-border-2);
    padding: 6px;
    border: 1px solid var(--color-border-2);
    border-radius: 4px;
    transition: all 0.1s ease-out;

    &:hover,
    &:focus {
      color: white;
      border-color: white;
    }
  }

  .code-block {
    margin: 18px 0;
    padding: 10px 18px;
    white-space: break-spaces;
    word-break: break-all;

    background: rgba(248, 250, 246, 0.03);

    border: 1px solid rgba(248, 250, 246, 0.1);
    border-radius: 4px;
  }

  .active {
    background: var(--color-secondary);
  }
</style>
