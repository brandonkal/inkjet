function __inkjet_dynamic_complete
    complete -c inkjet -e
    inkjet inkjet-dynamic-completions fish | source
end

# Set up the dynamic completion
complete -c inkjet -f -a "(__inkjet_dynamic_complete)"
