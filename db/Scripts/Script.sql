SELECT 
    sz.id AS schichtzuordnung_id,
    sz.mitarbeiter_idx,
    sz.schicht_id,
    s_focus.wache,
    s_focus.start,
    s_focus.ende,
    m_focus.schichtgruppe AS eigene_gruppe,
    COUNT(DISTINCT m_other.schichtgruppe) AS zeitgleiche_schichtgruppen
FROM schichtzuordnungs sz
JOIN schichts s_focus ON sz.schicht_id = s_focus.id
JOIN mitarbeiters m_focus ON sz.mitarbeiter_idx = m_focus.id
JOIN schichts s_other ON (
    s_other.start < s_focus.ende
    AND s_other.ende > s_focus.start
)
JOIN schichtzuordnungs sz_other ON s_other.id = sz_other.schicht_id
JOIN mitarbeiters m_other ON sz_other.mitarbeiter_idx = m_other.id
GROUP BY 
    sz.id, 
    sz.mitarbeiter_idx, 
    sz.schicht_id, 
    s_focus.wache, 
    s_focus.start, 
    s_focus.ende, 
    m_focus.schichtgruppe;
