import type { Workspace } from '../lib/use-workspace';

type Props = Pick<Workspace['scope'], 'projects' | 'selected' | 'setSelected'> & { busy: boolean };

export default function ProjectScope({ projects, selected, setSelected, busy }: Props) {
  return (
    <label className="scope-picker">
      Research project
      <select value={selected} disabled={busy} onChange={(e) => setSelected(e.target.value)}>
        <option value="">All projects</option>
        {projects.map((project) => (
          <option key={project.id} value={project.id}>
            {project.name}
          </option>
        ))}
      </select>
    </label>
  );
}
