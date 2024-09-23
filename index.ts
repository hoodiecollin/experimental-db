import * as prompt from '@inquirer/prompts';
import { Command } from 'commander';
import fs from 'node:fs/promises';

process.on('SIGINT', () => {
  console.log('Ctrl+C pressed');
  process.exit(0);
});

const program = new Command();

program
  .name('experimental-db-dev-tools')
  .description('Task management CLI tool powered by height.app API')
  .version('0.0.1');

program
  .command('list')
  .alias('lists')
  .description('Retrieve all tasks in a list (or multiple lists)')
  .argument('[query]', 'Search query');

program
  .command('task')
  .alias('tasks')
  .description('Retrieve a single task')
  .argument('<id>', 'Task ID');

await fs.mkdir('.tmp', { recursive: true });

let apiKey: string;

if (!(await fs.exists('.tmp/height-api-key'))) {
  apiKey = await prompt.password({
    message: 'Enter your Height API key',
  });

  await fs.writeFile('.tmp/height-api-key', apiKey);
} else {
  apiKey = await fs.readFile('.tmp/height-api-key', 'utf-8');
}

program.parse();

type ListModel = {
  id: string;
  model: 'list';
  type: 'list' | 'smartlist' | 'user' | 'inbox' | 'search';
  key: string;
  description: string;
  url: string;
  hue: number | null;
  visualization: 'list' | 'kanban' | 'calendar' | 'gannt';
};

async function list(query: string) {
  return await apiRequest<ListModel[]>('list');
}

type TaskModel = {
  id: string;
  model: 'task';
  index: number;
  listIds: string[];
  name: string;
  description: string;
  status: string[];
  parentTaskId: string | null;
  fields: any[];
};

async function task(id: string) {
  // fetch a single task
}

async function apiRequest<T = never>(
  pathname: string,
  params?: (q: URLSearchParams) => void | URLSearchParams
) {
  let suffix = '';

  if (params) {
    const q = new URLSearchParams();
    params(q);
    suffix = `?${q.toString()}`;
  }

  const res = await fetch(`https://api.height.app/${pathname}${suffix}`, {
    headers: {
      Authorization: `api-key ${apiKey}`,
    },
  });

  return res.json() as Promise<T>;
}
