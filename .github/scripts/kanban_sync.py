import os
import yaml
from github import Github
import re

def parse_kanban_md(file_path='KANBAN.md'):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Parse sections
    sections = {}
    current_section = None
    for line in content.split('\n'):
        if line.startswith('## '):
            current_section = line[3:].strip()
            sections[current_section] = []
        elif line.startswith('- [ ]') and current_section:
            task = line[5:].strip()
            sections[current_section].append({
                'title': task,
                'status': current_section
            })
        elif line.startswith('- [x]') and current_section:
            task = line[5:].strip()
            sections[current_section].append({
                'title': task,
                'status': 'Completed',
                'completed': True
            })

    return sections

def get_priority(task_title, content):
    # Extract priority from Priority Matrix section
    priority_match = re.search(r'### High Priority\n(.*?)###', content, re.DOTALL)
    if priority_match and task_title in priority_match.group(1):
        return 'High'
    return 'Normal'

def main():
    # Initialize GitHub client
    g = Github(os.getenv('GH_TOKEN'))
    repo = g.get_repo(os.getenv('GITHUB_REPOSITORY'))
    
    # Get or create project
    project_number = int(os.getenv('PROJECT_NUMBER'))
    
    # Using GraphQL to interact with Projects V2
    query = """
    mutation($projectId: ID!, $title: String!, $body: String) {
      createProjectV2Item(input: {projectId: $projectId, title: $title, body: $body}) {
        item {
          id
        }
      }
    }
    """

    # Parse KANBAN.md
    kanban_data = parse_kanban_md()

    # Create project columns if they don't exist
    status_fields = ['Backlog', 'Ready for Development', 'In Progress', 'Testing', 'Completed', 'Blocked']

    # Sync tasks
    for status, tasks in kanban_data.items():
        for task in tasks:
            # Create or update issue
            issue_title = task['title']
            existing_issues = repo.get_issues(state='all')
            
            issue = None
            for existing_issue in existing_issues:
                if existing_issue.title == issue_title:
                    issue = existing_issue
                    break

            if not issue:
                issue = repo.create_issue(
                    title=issue_title,
                    body=f"Task from Kanban board\nStatus: {status}",
                    labels=[status, get_priority(issue_title, open('KANBAN.md').read())]
                )

            # Update project card
            if task.get('completed'):
                issue.edit(state='closed')

if __name__ == '__main__':
    main()