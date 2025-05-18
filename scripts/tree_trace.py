import sys
from collections import defaultdict

def parse_dot(filename):
    graph = defaultdict(list)
    labels = {}
    with open(filename) as f:
        for line in f:
            line = line.strip()
            if "->" in line:
                parts = line.replace(";", "").replace('"', "").split("->")
                src, dst = [p.strip() for p in parts]
                graph[src].append(dst)
            elif "[" in line and "label=" in line:
                # node label
                parts = line.split("[label=")
                node = parts[0].replace('"', '').strip()
                label = parts[1].split(']')[0].strip(' "\'')
                labels[node] = label
    return graph, labels

def dfs(graph, start, path, paths):
    path = path + [start]
    if not graph.get(start):
        paths.append(path)
        return
    for neighbor in graph[start]:
        if neighbor in path:
            continue  # avoid cycles
        dfs(graph, neighbor, path, paths)

def main():
    if len(sys.argv) != 3:
        print("Usage: python3 tree_trace.py <dot file> <start node>")
        sys.exit(1)
    dotfile = sys.argv[1]
    entry = sys.argv[2]
    graph, labels = parse_dot(dotfile)
    paths = []
    dfs(graph, entry, [], paths)
    print(f"=== All call traces from {entry} ===")
    for i, p in enumerate(paths, 1):
        print(f"Path {i}:")
        for node in p:
            print(f"  {node} [{labels.get(node,'')}]")
        print("")

if __name__ == "__main__":
    main()
