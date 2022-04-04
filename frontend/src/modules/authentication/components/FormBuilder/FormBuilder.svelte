<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';

  import Input from 'modules/forms/components/Input/Input.svelte';

  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { Hint } from 'svelte-use-form';

  export let nodes = [];
  export let form;
  export let validationMapper;
  export let hasPasswordConfirm = false;

  let isSubmitting = false;

  const handleSubmit = () => (isSubmitting = true);

  let type = 'password';

  const handleToggle = (e) => {
    e.preventDefault();
    const newType = type === 'password' ? 'text' : 'password';
    type = newType;
  };
</script>

{#each nodes as { attributes, meta, ...node }}
  {#if attributes.name === 'csrf_token'}
    <input
      {...node}
      data-testid="auth-csrf"
      bind:value={attributes.value}
      type="hidden"
      name={attributes.name}
    />
  {:else if attributes.type === 'submit' && attributes.name !== 'provider'}
    <li class="register__input-item--submit">
      <Button
        on:click={handleSubmit}
        size="medium"
        display="block"
        type="submit"
        name={attributes.name}
        value={attributes.value}
      >
        {#if isSubmitting}
          &nbsp;
          <LoadingSpinner size="button" id="js-form-submit" />
        {:else}
          <slot name="submit-button-label">Submit</slot>
        {/if}
      </Button>
    </li>
  {:else if attributes.name === 'password'}
    <li class="register__input-item">
      <PasswordField
        {...node}
        field={$form?.[attributes.name]}
        name={attributes.name}
        validate={validationMapper[attributes.name]}
        placeholder={meta?.label?.text}
        labelClass="visually-hidden"
        {type}
      >
        <PasswordToggle
          on:click={handleToggle}
          activeType={type}
          slot="utilRight"
        />
        <svelte:fragment slot="label">Password label</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="passwordMatch">Passwords do not match</Hint>
        </svelte:fragment>
      </PasswordField>
    </li>
    {#if hasPasswordConfirm}
      <li class="register__input-item">
        <PasswordField
          field={$form?.confirmPassword}
          name="confirmPassword"
          placeholder="Confirm Password"
          labelClass="visually-hidden"
          validate={validationMapper.confirmPassword}
          {type}
        >
          <PasswordToggle
            on:click={handleToggle}
            activeType={type}
            slot="utilRight"
          />
          <svelte:fragment slot="label">Confirm Password</svelte:fragment>
          <svelte:fragment slot="hints">
            <Hint on="required">This is a mandatory field</Hint>
            <Hint on="passwordMatch">Passwords do not match</Hint>
          </svelte:fragment>
        </PasswordField>
      </li>
    {/if}
  {:else}
    <li class="register__input-item">
      <Input
        field={$form[attributes.name]}
        name={attributes.name}
        placeholder={meta?.label?.text}
        labelClass="visually-hidden"
        validate={validationMapper[attributes.name]}
        {...node}
      >
        <svelte:fragment slot="label">{node?.meta?.label?.text}</svelte:fragment
        >
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
    </li>
  {/if}
{/each}

<style>
  .register__input-item {
    margin-bottom: 12px;
  }

  .register__input-item--submit {
    margin-top: 24px;

    & :global(button) {
      position: relative;
    }
  }
</style>
