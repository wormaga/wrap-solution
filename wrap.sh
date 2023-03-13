#!/usr/bin/env bash


#rust install command
#curl https://sh.rustup.rs -sSf | sh -s -- --help


echo "Welcome to wrap v1.0"

echo "Checking if Rust Lang is installed"

isRustInstalled = $(which cargo)

if which cargo ; then
    echo "Rust is installed"
else
    echo "Installing Rust Lang..."
    curl https://sh.rustup.rs -sSf | sh -s -- --help

    if which cargo ; then
        echo "Rust is installed"
    else
        echo "Something went wrong"
        exit 1
    fi
fi



#get json with all available cli tools

echo ""
#ask user what tools they need
names=(Donald Alan Brian)
selected=()
nulls='Select tools you want to install (type space seperated numbers): '
select name in "${names[@]}" ; do
    for reply in $REPLY ; do
        selected+=(${names[reply - 1]})
    done
    [[ $selected ]] && break
done


echo Selected: "${selected[@]}"

echo ""
#install there tools
arrayLength=${#selected[@]}
echo "array L: $arrayLength"
for (( i=0; i<${arrayLength}; i++))
do
    echo "value ${selected[$i]}"
    echo "doing something smart"
done













echo ""
echo "Exiting"
echo ""

## date -u +"%Y-%m-%dT%H:%M:%SZ"
# {
#     "lastUpdate" : "2021-01-17T04:16:14Z",
#     "tools": [
#         {
#             "name": "litegallery",
#             "version": 1.3, #check this
#             "files"@ [
#                 {
#                     "location": "${HOME}/.cli-tools/litegallery/Cargo.toml",
#                     "url": "https://github.com/wormaga/.../.../.../Cargo.toml"                    
#                 },
#                 {
#                     "location": "${HOME}/.cli-tools/litegallery/src/main.rs",
#                     "url": "https://github.com/wormaga/.../.../.../main.rs"
#                 }
#             ]
#         }
#     ]

# }