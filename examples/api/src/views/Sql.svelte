<script>
  import Database from '@tauri-apps/plugin-sql'
  import { onMount } from 'svelte'

  export let onMessage

  // the `todos` table is created by the migration the app registers for this database
  let db
  let todos = []
  let title = ''

  async function refresh() {
    todos = await db.select('SELECT * FROM todos ORDER BY id')
  }

  onMount(async () => {
    try {
      db = await Database.load('sqlite:api.db')
      await refresh()
    } catch (error) {
      onMessage(error)
    }
  })

  function add() {
    db.execute('INSERT INTO todos (title) VALUES ($1)', [title])
      .then((result) => {
        onMessage(result)
        title = ''
        return refresh()
      })
      .catch(onMessage)
  }

  function toggle(todo) {
    db.execute('UPDATE todos SET done = $1 WHERE id = $2', [
      todo.done ? 0 : 1,
      todo.id
    ])
      .then(refresh)
      .catch(onMessage)
  }

  function remove(todo) {
    db.execute('DELETE FROM todos WHERE id = $1', [todo.id])
      .then(refresh)
      .catch(onMessage)
  }
</script>

<div class="flex flex-col gap-2">
  <form class="flex flex-row gap-2" on:submit|preventDefault={add}>
    <input class="input grow" placeholder="New todo" bind:value={title} />
    <button class="btn" type="submit" disabled={!db || !title}>Add</button>
  </form>

  {#each todos as todo (todo.id)}
    <div class="flex flex-row gap-2 items-center">
      <input
        type="checkbox"
        checked={todo.done === 1}
        on:change={() => toggle(todo)}
      />
      <span class="grow" class:line-through={todo.done === 1}>{todo.title}</span>
      <button class="btn" on:click={() => remove(todo)}>Delete</button>
    </div>
  {/each}
</div>
