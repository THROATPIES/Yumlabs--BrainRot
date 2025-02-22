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
        elif (line.startswith('- [ ]') or line.startswith('- [x]')) and current_section:
            completed = line.startswith('- [x]')
            task = line[5:].strip()
            sections[current_section].append({
                'title': task,
                'status': 'Completed' if completed else current_section,
                'completed': completed
            })

    return sections

def get_priority(task_title, content):
    priority_sections = {
        'High Priority': 'high-priority',
        'Medium Priority': 'medium-priority',
        'Low Priority': 'low-priority'
    }
    
    for section, label in priority_sections.items():
        section_match = re.search(f'### {section}\n(.*?)(?=###|\Z)', content, re.DOTALL)
        if section_match and task_title in section_match.group(1):
            return label
    
    return 'medium-priority'  # default priority

def sync_issues(repo, kanban_data):
    # Get all existing issues
    existing_issues = list(repo.get_issues(state='all'))
    
    # Track processed issues to handle deletions
    processed_issues = set()
    
    for status, tasks in kanban_data.items():
        for task in tasks:
            issue_title = task['title']
            
            # Find existing issue
            existing_issue = next(
                (issue for issue in existing_issues if issue.title == issue_title),
                None
            )
            
            # Get priority label
            priority = get_priority(issue_title, open('KANBAN.md').read())
            labels = [status, priority]
            
            if existing_issue:
                # Update existing issue
                processed_issues.add(existing_issue.number)
                
                # Update state
                new_state = 'closed' if task['completed'] else 'open'
                if existing_issue.state != new_state:
                    existing_issue.edit(state=new_state)
                
                # Update labels
                current_labels = [label.name for label in existing_issue.labels]
                if set(labels) != set(current_labels):
                    existing_issue.edit(labels=labels)
                
            else:
                # Create new issue
                issue = repo.create_issue(
                    title=issue_title,
                    body=f"Task from Kanban board\nStatus: {status}",
                    labels=labels
                )
                processed_issues.add(issue.number)
                
                if task['completed']:
                    issue.edit(state='closed')
    
    # Close issues that are no longer in KANBAN.md
    for issue in existing_issues:
        if (issue.number not in processed_issues and 
            not issue.pull_request and  # Skip PRs
            issue.state == 'open'):     # Only close open issues
            issue.edit(state='closed')

def main():
    # Initialize GitHub client
    g = Github(os.getenv('GH_TOKEN'))
    repo = g.get_repo(os.getenv('GITHUB_REPOSITORY'))
    
    # Parse KANBAN.md
    kanban_data = parse_kanban_md()
    
    # Sync issues
    sync_issues(repo, kanban_data)

if __name__ == "__main__":
    main()
