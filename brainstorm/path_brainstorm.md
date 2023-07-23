get file pairs(root, original, changes, current_path default = "")
    assumptions: 
    - root is a directory
    - original and changes lead to a directories

    res = []
    loop through root/original
        if path is file
            find file with same name under root/changes/{path.name}
        else if path is dir
            new_path = current_path.clone()
            new_path.push(path)
            get file pairs(root, original, changes, new_path)