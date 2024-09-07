local in_section_to_remove = false
local section_level = 0

function Header(el)
  if el.content[1].text == "Installation" or el.content[1].text == "Motivation" then
    in_section_to_remove = true
    section_level = el.level
    return {}
  elseif in_section_to_remove and el.level > section_level then
    return {}
  elseif in_section_to_remove and el.level <= section_level then
    in_section_to_remove = false
  end
end

function Block(el)
  if in_section_to_remove then
    return {}
  end
end
