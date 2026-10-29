SELECT *
FROM schichtzuordnungs sz
JOIN mitarbeiters m ON sz.mitarbeiter_idx = m."index" 
GROUP BY m."index" 
