'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'] & Workspace['scope'] & Workspace['projectForm'];

export default function Projects({
  user,
  busy,
  act,
  projects,
  selected,
  setSelected,
  projectName,
  setProjectName,
}: Props) {
  return (
    <div className="subpanel">
      <span className="eyebrow">YOUR PROJECTS</span>
      <div className="row">
        <input
          aria-label="Project name"
          placeholder="New research project"
          value={projectName}
          onChange={(e) => setProjectName(e.target.value)}
        />
        <button
          className="tiny"
          disabled={!user || busy || !projectName.trim()}
          onClick={() =>
            void act(async () => {
              await api('/projects', { method: 'POST', body: JSON.stringify({ name: projectName }) });
              setProjectName('');
            }, 'Project created')
          }
        >
          Add
        </button>
      </div>
      <div className="list">
        {projects.map((project) => (
          <div className="list-item job" key={project.id}>
            <b>{project.name}</b>
            <button
              className="tiny"
              disabled={busy}
              onClick={() => {
                if (
                  window.confirm(
                    'Delete this project? Its evidence and reports remain available as unassigned records.',
                  )
                )
                  void act(async () => {
                    await api('/projects/' + project.id, { method: 'DELETE' });
                    if (selected === project.id) setSelected('');
                  }, 'Project deleted; research was preserved');
              }}
            >
              Delete
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
